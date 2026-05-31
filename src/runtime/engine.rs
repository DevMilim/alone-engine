use std::{any::Any, collections::VecDeque, time::Instant};

use indexmap::IndexMap;

use crate::{
    AudioSys, Base, CollisionWorld, DrawCommand, EngineContext, GameObjectDispatch, GlobalEvent,
    Handler, Id, ImageAsset, InputState, InputType, RenderQueue, Resources, RuntimeCommands, Scene,
    TriggerEvent, TriggerKind, Vector2,
};

pub const FIXED_DT: f32 = 1.0 / 60.0;
const MAX_ACCUM: f32 = 0.5;

pub struct Engine<S: Scene> {
    collision: CollisionWorld,
    objects: Vec<S>,
    base: Base,
    is_running: bool,
    pub input: InputState,
    pub event_queue: VecDeque<RuntimeCommands>,
    events: VecDeque<GlobalEvent>,
    mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,
    pub camera_pos: Vector2,
    pub resources: Resources,
    last_instant: Instant,
    accumulator: f32,
    audio_sys: AudioSys,
}

impl<S: Scene> Engine<S> {
    pub fn new(main_scene: S) -> Self {
        Self {
            resources: Resources::new(),
            objects: vec![main_scene],
            base: Base::default(),
            is_running: true,
            input: InputState::new(),
            event_queue: VecDeque::new(),
            events: VecDeque::new(),
            mailbox: IndexMap::new(),
            camera_pos: Vector2::ZERO,
            last_instant: Instant::now(),
            accumulator: 0.0,
            collision: CollisionWorld::new(),
            audio_sys: AudioSys::new(),
        }
    }
    pub fn events(&mut self, cmd: RuntimeCommands) {
        match cmd {
            RuntimeCommands::Quit => self.is_running = false,
            RuntimeCommands::KeyDown(keycode) => {
                self.input.update_input_state(InputType::Key(keycode), true)
            }
            RuntimeCommands::KeyUp(keycode) => self
                .input
                .update_input_state(InputType::Key(keycode), false),
            RuntimeCommands::MousePosition(x, y) => self.input.set_mouse_position(x, y),
            RuntimeCommands::MouseInput(mouse_button, is_pressed) => self
                .input
                .update_input_state(InputType::Mouse(mouse_button), is_pressed),
        }
    }

    pub fn render(&mut self, renderer: &mut [Vec<DrawCommand>; 6], blending: f32) {
        let mut render_ctx = RenderQueue {
            queue: renderer,
            camera: &mut self.camera_pos,
        };
        if let Some(obj) = self.objects.last_mut() {
            obj.get_dispatch()
                .dispatch_draw(&mut render_ctx, &self.base, blending);
        } else {
            self.quit();
        }
    }

    pub fn get_texture(&mut self, handler: Handler<ImageAsset>) -> Option<&ImageAsset> {
        self.resources.textures.get(handler)
    }

    pub fn update_step(&mut self) -> (bool, f32) {
        let now = Instant::now();
        let mut delta_time = (now - self.last_instant).as_secs_f32();
        self.last_instant = now;

        if delta_time >= MAX_ACCUM {
            delta_time = MAX_ACCUM;
        }

        self.accumulator += delta_time;
        let blending = self.update(delta_time);

        self.flush_messages_and_events();
        self.collision.step();
        self.collision_step();

        self.collision.commit();
        self.input.clear_frame_data();
        (self.is_running, blending)
    }
    pub fn push(&mut self, scene: S) {
        self.objects.push(scene);
    }
    pub fn pop(&mut self) {
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut self.collision,
            &self.audio_sys,
        );
        if let Some(mut scene) = self.objects.pop() {
            scene.get_dispatch().dispatch_destroy(&mut ctx);
        }
    }
    pub fn set_scene(&mut self, scene: S) {
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut self.collision,
            &self.audio_sys,
        );
        while let Some(mut old_scene) = self.objects.pop() {
            old_scene.get_dispatch().dispatch_destroy(&mut ctx);
        }
        self.objects.clear();
        self.objects.push(scene);
    }

    pub fn collision_step(&mut self) {
        let ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut self.collision,
            &self.audio_sys,
        );
        for (a, b) in ctx.collision.get_entered_pairs() {
            let da = ctx.collision.colliders.get(&a).unwrap();
            let db = ctx.collision.colliders.get(&b).unwrap();

            if da.is_sensor {
                let ev = TriggerEvent {
                    owner: b.id,
                    sensor: a.clone(),
                    kind: TriggerKind::Enter,
                };
                ctx.events
                    .push_back(GlobalEvent::Targeted(a.id, Box::new(ev.clone())));
                ctx.events
                    .push_back(GlobalEvent::Targeted(b.id, Box::new(ev)));
            }
            if db.is_sensor {
                let ev = TriggerEvent {
                    owner: a.id,
                    sensor: b.clone(),
                    kind: TriggerKind::Enter,
                };
                ctx.events
                    .push_back(GlobalEvent::Targeted(b.id, Box::new(ev.clone())));
                ctx.events
                    .push_back(GlobalEvent::Targeted(a.id, Box::new(ev)));
            }
        }
        for (a, b) in ctx.collision.get_exited_pairs() {
            let da = ctx.collision.colliders.get(&a).unwrap();
            let db = ctx.collision.colliders.get(&b).unwrap();

            if da.is_sensor {
                let ev = TriggerEvent {
                    owner: b.id,
                    sensor: a.clone(),
                    kind: TriggerKind::Exit,
                };
                ctx.events
                    .push_back(GlobalEvent::Targeted(a.id, Box::new(ev.clone())));
                ctx.events
                    .push_back(GlobalEvent::Targeted(b.id, Box::new(ev)));
            }
            if db.is_sensor {
                let ev = TriggerEvent {
                    owner: a.id,
                    sensor: b.clone(),
                    kind: TriggerKind::Exit,
                };
                ctx.events
                    .push_back(GlobalEvent::Targeted(b.id, Box::new(ev.clone())));
                ctx.events
                    .push_back(GlobalEvent::Targeted(a.id, Box::new(ev)));
            }
        }
    }

    pub fn update(&mut self, delta_time: f32) -> f32 {
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut self.collision,
            &self.audio_sys,
        );
        if let Some(obj) = self.objects.last_mut() {
            obj.get_dispatch()
                .dispatch_update(&mut ctx, &self.base, delta_time);
        }
        while self.accumulator >= FIXED_DT {
            if let Some(obj) = self.objects.last_mut() {
                obj.get_dispatch()
                    .dispatch_fixed_update(&mut ctx, &self.base, FIXED_DT);
            }
            self.accumulator -= FIXED_DT;
        }

        if let Some(obj) = self.objects.last_mut() {
            obj.get_dispatch()
                .dispatch_late_update(&mut ctx, &self.base, delta_time);
        }
        self.accumulator / FIXED_DT
    }
    pub fn flush_messages_and_events(&mut self) {
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut self.collision,
            &self.audio_sys,
        );
        for _ in 0..10 {
            let mut something_processed = false;
            while let Some(event) = &ctx.events.pop_front() {
                something_processed = true;
                if let Some(obj) = self.objects.last_mut() {
                    obj.get_dispatch().dispatch_event(&mut ctx, event);
                }
            }
            if !ctx.mailbox.is_empty() {
                something_processed = true;
                if let Some(obj) = self.objects.last_mut() {
                    obj.get_dispatch().dispatch_message(&mut ctx);
                }
            }
            if !something_processed {
                break;
            }
        }
    }
    pub fn quit(&mut self) {
        self.is_running = false;
    }

    pub fn clear_unecessary(&mut self) {
        self.resources.clear();
    }
}
