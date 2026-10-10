use std::{
    any::Any,
    sync::Arc,
    time::{Duration, Instant},
};

use gilrs::{
    EventType, GamepadId, Gilrs,
    ff::{BaseEffect, BaseEffectType, EffectBuilder, Repeat, Replay, Ticks},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use crate::{
    core::{Base, CoreSystems},
    event::{EventManager, GlobalEvent},
    input::InputType,
    math::Vector2,
    render::{LOGICAL_HEIGHT, LOGICAL_WIDTH, Render},
    rng::Random,
    runtime::{EmptyGlobals, EngineContext, GameObjectDispatch, Scene, State, WorldState},
};

#[derive(Debug)]
pub enum AppCommands {
    ChangeScene(Box<dyn Any>),
    PushScene(Box<dyn Any>),
    ClearScenes,
    PopScene,
}

pub struct App<S: Scene + 'static, P: GameObjectDispatch = EmptyGlobals> {
    pub systems: CoreSystems,
    pub events: EventManager,
    pub world: WorldState<S, P>,
    pub render: Option<Render<'static>>,
    pub state: State,

    pub window: Option<Arc<Window>>,
    pub base: Base,
    pub camera_position: Vector2,
    pub update_frame_count: u64,
    pub fixed_frame_count: u64,
    pub rng: Random,
    pub gilrs: Option<Gilrs>,
    pub active_effects: Vec<(gilrs::ff::Effect, Instant)>,
    pub continuous_effect: Option<(gilrs::ff::Effect, GamepadId)>,
    pub continuous_gain: f32,
}

impl<S: Scene + 'static, P: GameObjectDispatch> App<S, P> {
    pub fn new(root_scene: S) -> Self {
        Self {
            systems: CoreSystems::default(),
            events: EventManager::default(),
            world: WorldState::new(root_scene),
            state: State::new(),
            render: None,
            window: None,
            base: Base::default(),
            camera_position: Vector2::new(0.0, 0.0),
            update_frame_count: 0,
            fixed_frame_count: 0,
            rng: Random::time_seed(),
            gilrs: Gilrs::new()
                .map_err(|e| eprintln!("gamepad indisponível: {e}"))
                .ok(),
            active_effects: Vec::new(),
            continuous_effect: None,
            continuous_gain: 0.0,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }
    pub fn with_globals(&mut self, global: P) -> &mut Self {
        self.world.global = Some(global);
        self
    }
    fn poll_gamepads(&mut self) {
        let Some(gilrs) = &mut self.gilrs else { return };
        let input = &mut self.systems.input;

        if input.active_gamepad.is_none() {
            input.active_gamepad = gilrs.gamepads().next().map(|(id, _)| id);
        }

        while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            if matches!(event, EventType::ButtonPressed(..)) && input.active_gamepad != Some(id) {
                input.release_gamepad();
                input.active_gamepad = Some(id);
            }

            match event {
                EventType::Connected if input.active_gamepad.is_none() => {
                    input.active_gamepad = Some(id);
                }
                EventType::Disconnected if input.active_gamepad == Some(id) => {
                    input.release_gamepad();
                    input.active_gamepad = gilrs.gamepads().map(|(g, _)| g).find(|&g| g != id);
                }
                _ if input.active_gamepad != Some(id) => {}
                EventType::ButtonPressed(b, _) => {
                    input.update_input_state(InputType::Gamepad(b), true)
                }
                EventType::ButtonReleased(b, _) => {
                    input.update_input_state(InputType::Gamepad(b), false)
                }
                EventType::AxisChanged(axis, value, _) => input.update_axis(axis, value),
                EventType::ButtonChanged(b, value, _) => input.update_button_value(b, value),
                _ => {}
            }
        }
    }
    fn process_rumble(&mut self) {
        let now = Instant::now();
        self.active_effects.retain(|(_, end)| *end > now);

        let requests = std::mem::take(&mut self.systems.input.rumble_requests);
        let Some(gilrs) = &mut self.gilrs else { return };
        let Some(id) = self.systems.input.active_gamepad else {
            return;
        };
        if !gilrs.gamepad(id).is_ff_supported() {
            return;
        }

        let to_u16 = |v: f32| (v.clamp(0.0, 1.0) * u16::MAX as f32) as u16;

        for r in requests {
            let scheduling = Replay {
                play_for: Ticks::from_ms(r.duration_ms),
                ..Default::default()
            };

            let effect = EffectBuilder::new()
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Strong {
                        magnitude: to_u16(r.strong),
                    },
                    scheduling,
                    envelope: Default::default(),
                })
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Weak {
                        magnitude: to_u16(r.weak),
                    },
                    scheduling,
                    envelope: Default::default(),
                })
                .gamepads(&[id])
                .finish(gilrs);

            match effect {
                Ok(effect) => {
                    if effect.play().is_ok() {
                        let end = now + Duration::from_millis(r.duration_ms as u64 + 50);
                        self.active_effects.push((effect, end));
                    }
                }
                Err(e) => eprintln!("falha ao criar vibração: {e}"),
            }
        }
    }
    fn process_continuous_rumble(&mut self) {
        let intensity = self.systems.input.continuous_rumble;
        let active = self.systems.input.active_gamepad;
        let Some(gilrs) = &mut self.gilrs else { return };

        // Soltou o gatilho ou trocou de controle: dropar o Effect para a vibração
        let wrong_pad = matches!(&self.continuous_effect, Some((_, id)) if Some(*id) != active);
        if intensity <= 0.0 || wrong_pad {
            self.continuous_effect = None;
            self.continuous_gain = 0.0;
            if intensity <= 0.0 {
                return;
            }
        }

        let Some(id) = active else { return };
        if !gilrs.gamepad(id).is_ff_supported() {
            return;
        }

        // Cria o efeito na primeira vez que precisar
        if self.continuous_effect.is_none() {
            let scheduling = Replay {
                play_for: Ticks::from_ms(1000),
                ..Default::default()
            };
            let built = EffectBuilder::new()
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Strong {
                        magnitude: u16::MAX,
                    },
                    scheduling,
                    envelope: Default::default(),
                })
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Weak {
                        magnitude: u16::MAX / 2,
                    },
                    scheduling,
                    envelope: Default::default(),
                })
                .repeat(Repeat::Infinitely)
                .gamepads(&[id])
                .finish(gilrs);

            match built {
                Ok(effect) => {
                    let _ = effect.set_gain(intensity);
                    if effect.play().is_ok() {
                        self.continuous_gain = intensity;
                        self.continuous_effect = Some((effect, id));
                    }
                }
                Err(e) => eprintln!("falha ao criar vibração contínua: {e}"),
            }
            return;
        }

        // Só atualiza quando a mudança for perceptível, para não mandar comando todo frame
        if let Some((effect, _)) = &self.continuous_effect {
            if (intensity - self.continuous_gain).abs() > 0.01 {
                let _ = effect.set_gain(intensity);
                self.continuous_gain = intensity;
            }
        }
    }
}

impl<S: Scene + 'static, P: GameObjectDispatch> ApplicationHandler for App<S, P> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("Alone Engine")
            .with_inner_size(LogicalSize::new(800, 600));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        self.render = Some(Render::new(&window));
        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.render.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    self.systems
                        .input
                        .update_input_state(InputType::Key(keycode), event.state.is_pressed());
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.systems
                    .input
                    .update_input_state(InputType::Mouse(button), state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Ok(mouse) = self
                    .render
                    .as_mut()
                    .unwrap()
                    .pixels
                    .window_pos_to_pixel((position.x as f32, position.y as f32))
                {
                    let camera = self.camera_position;
                    self.systems
                        .input
                        .set_mouse_position(mouse.0 as f32 + camera.x, mouse.1 as f32 + camera.y);
                }
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                let _ = state.pixels.resize_surface(size.width, size.height);

                state.set_window_size((LOGICAL_WIDTH, LOGICAL_HEIGHT));

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.poll_gamepads();

        self.update_frame_count += 1;
        self.systems.input.current_update_frame = self.update_frame_count;
        self.systems.input.current_fixed_frame = self.fixed_frame_count;

        let Some(render) = &mut self.render else {
            panic!("Erro ao obter render")
        };

        let mut ctx = EngineContext {
            systems: &mut self.systems,
            events: &mut self.events,
            camera_position: &mut self.camera_position,
            window_size: &render.window_size,
            is_fixed_update: false,
            state: &mut self.state,
            rng: &mut self.rng,
        };

        let (is_running, blending) =
            self.world
                .update(&mut ctx, &self.base, &mut self.fixed_frame_count);

        while let Ok(bg_event) = ctx.systems.bg_event_receiver.try_recv() {
            match bg_event {
                GlobalEvent::Broadcast(any) => ctx.events.insert_broadcast_event(any),
                other => ctx.events.insert_global_event(other),
            }
        }

        const MAX_EVENT_ROUNDS: u32 = 10;
        let mut last_broadcast_version = 0;

        for round in 0..MAX_EVENT_ROUNDS {
            let mailboxes_pending = !ctx.events.mailboxes.is_empty();
            let broadcasts_pending = ctx.events.broadcast_version != last_broadcast_version;

            if !mailboxes_pending && !broadcasts_pending {
                break;
            }
            last_broadcast_version = ctx.events.broadcast_version;

            if let Some(global) = &mut self.world.global {
                global.dispatch_events(&mut ctx);
            }
            self.world.last_scene().dispatch_events(&mut ctx);

            ctx.events.prune_dead_mailboxes(&ctx.systems.live_ids);

            let still_pending = !ctx.events.mailboxes.is_empty()
                || ctx.events.broadcast_version != last_broadcast_version;

            if round == MAX_EVENT_ROUNDS - 1 && still_pending {
                eprintln!("limite de rounds de evento atingido, possível loop de eventos");
            }
        }

        ctx.events.clear_broadcast_log();

        while let Some(cmd) = ctx.events.aplication_commands.pop_front() {
            match cmd {
                AppCommands::ChangeScene(scene) => {
                    if let Ok(scene) = scene.downcast::<S>() {
                        self.world.change_scene(*scene, &mut ctx);
                    }
                }
                AppCommands::PushScene(scene) => {
                    if let Ok(scene) = scene.downcast::<S>() {
                        self.world.push_scene(*scene);
                    }
                }
                AppCommands::PopScene => {
                    self.world.pop_scene(&mut ctx);
                }
                AppCommands::ClearScenes => {
                    self.world.clear_scenes(&mut ctx);
                }
            }
        }

        self.world
            .render(&mut render.queue, &mut ctx, &self.base, blending);

        render.render_auto(self.camera_position, &self.systems.resources);

        if !is_running {
            event_loop.exit();
        }

        self.window.as_mut().unwrap().request_redraw();

        self.process_rumble();
        self.process_continuous_rumble();

        if !is_running {
            event_loop.exit();
        }

        self.systems.input.clear_frame_data();
    }
}
