use std::{any::Any, collections::VecDeque, time::Instant};

use indexmap::IndexMap;

use crate::{
    Base, CollisionWorld, EngineContext, GameObjectDispatch, GlobalEvent, Handler, Id, ImageAsset,
    InputState, RenderApi, RenderCommands, Resources, Scene, Transform2D, TriggerEvent,
    TriggerKind, Vector2,
};

const FIXED_DT: f32 = 1.0 / 60.0;
const MAX_ACCUM: f32 = 0.5;

pub struct Engine<S: Scene> {
    collision: CollisionWorld,
    objects: Vec<S>,
    base: Base,
    is_running: bool,
    pub input: InputState,
    pub event_queue: VecDeque<RenderCommands>,
    events: VecDeque<GlobalEvent>,
    mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,
    pub camera_pos: Vector2,
    pub resources: Resources,
    last_instant: Instant,
    accumulator: f32,
}

impl<S: Scene> Engine<S> {
    pub fn new(main_scene: S) -> Self {
        Self {
            resources: Resources::new(),
            objects: vec![main_scene],
            base: Base::new(Transform2D::new(0.0, 0.0)),
            is_running: true,
            input: InputState::new(),
            event_queue: VecDeque::new(),
            events: VecDeque::new(),
            mailbox: IndexMap::new(),
            camera_pos: Vector2::ZERO,
            last_instant: Instant::now(),
            accumulator: 0.0,
            collision: CollisionWorld::new(),
        }
    }
    pub fn events(&mut self, cmd: RenderCommands) {
        match cmd {
            RenderCommands::Quit => self.is_running = false,
            RenderCommands::KeyDown(keycode) => self.input.update_key(keycode, true),
            RenderCommands::KeyUp(keycode) => self.input.update_key(keycode, false),
            RenderCommands::MousePosition(x, y) => self.input.set_mouse_position(x, y),
        }
    }

    pub fn render(&mut self, renderer: &mut impl RenderApi, blending: f32) {
        if let Some(obj) = self.objects.last_mut() {
            obj.get_dispatch()
                .dispatch_draw(renderer, &self.base, blending);
        } else {
            self.quit();
        }
    }

    pub fn get_texture(&self, handler: Handler<ImageAsset>) -> Option<&ImageAsset> {
        self.resources.textures.get(handler)
    }

    pub fn update_step(&mut self) -> (bool, f32) {
        self.input.clear_frame_data();

        let now = Instant::now();
        let mut delta_time = (now - self.last_instant).as_secs_f32();
        self.last_instant = now;

        if delta_time >= MAX_ACCUM {
            delta_time = MAX_ACCUM;
        }

        self.accumulator += delta_time;
        let blending = self.update(delta_time);

        self.flush_messages_and_events();

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
        );
        for (a, b) in ctx.collision.get_entered_pairs() {
            let da = ctx.collision.colliders.get(&a).unwrap();
            let db = ctx.collision.colliders.get(&b).unwrap();

            if da.is_sensor {
                let ev = TriggerEvent {
                    owner: b.id,
                    sensor: (a.key, a.id),
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
                    sensor: (b.key, b.id),
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
                    sensor: (a.key, a.id),
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
                    sensor: (b.key, b.id),
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
        );
        if let Some(obj) = self.objects.last_mut() {
            obj.get_dispatch()
                .dispatch_update(&mut ctx, &self.base, delta_time);
        }
        while self.accumulator > FIXED_DT {
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
