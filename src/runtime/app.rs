use std::{any::Any, sync::Arc};

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
    event::{BackGroundEvent, EventManager, GlobalEvent},
    input::InputType,
    math::Vector2,
    render::{LOGICAL_HEIGHT, LOGICAL_WIDTH, Render},
    runtime::{EmptyGlobals, EngineContext, GameObjectDispatch, Scene, WorldState},
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

    pub window: Option<Arc<Window>>,
    pub base: Base,
    pub camera_position: Vector2,
    pub update_frame_count: u64,
    pub fixed_frame_count: u64,
}

impl<S: Scene + 'static, P: GameObjectDispatch> App<S, P> {
    pub fn new(root_scene: S) -> Self {
        Self {
            systems: CoreSystems::default(),
            events: EventManager::default(),
            world: WorldState::new(root_scene),
            render: None,
            window: None,
            base: Base::default(),
            camera_position: Vector2::new(0.0, 0.0),
            update_frame_count: 0,
            fixed_frame_count: 0,
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
}

impl<S: Scene + 'static, P: GameObjectDispatch> ApplicationHandler for App<S, P> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("winit + pixels")
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
        };

        let (is_running, blending) =
            self.world
                .update(&mut ctx, &self.base, &mut self.fixed_frame_count);

        while let Ok(bg_event) = ctx.systems.bg_event_receiver.try_recv() {
            match bg_event {
                BackGroundEvent::Broadcast(event) => {
                    ctx.events
                        .global_events
                        .push_back(GlobalEvent::Broadcast(event));
                }
                BackGroundEvent::Targeted(id, event) => {
                    ctx.events
                        .global_events
                        .push_back(GlobalEvent::Targeted(id, event));
                }
                BackGroundEvent::Send(id, message) => {
                    ctx.events.mailbox.entry(id).or_default().push(message);
                }
            }
        }

        const MAX_EVENT_ROUNDS: u32 = 10;
        for round in 0..MAX_EVENT_ROUNDS {
            let mut something_processed = false;
            while let Some(event) = ctx.events.global_events.pop_front() {
                something_processed = true;
                if let Some(global) = &mut self.world.global {
                    global.dispatch_event(&mut ctx, &event);
                }
                self.world.last_scene().dispatch_event(&mut ctx, &event);
            }
            if !ctx.events.mailbox.is_empty() {
                something_processed = true;
                if let Some(global) = &mut self.world.global {
                    global.dispatch_message(&mut ctx);
                }
                self.world.last_scene().dispatch_message(&mut ctx);
            }
            if !something_processed {
                break;
            }
            if round == MAX_EVENT_ROUNDS - 1 && something_processed {
                eprintln!("limite de rounds de evento atingido, possivel loop de eventos")
            }
        }

        ctx.events
            .mailbox
            .retain(|id, _| ctx.systems.live_ids.contains(id));

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

        render.render(self.camera_position, &self.systems.resources);

        if !is_running {
            event_loop.exit();
        }
        self.window.as_mut().unwrap().request_redraw();
        self.systems.input.clear_frame_data();
    }
}
