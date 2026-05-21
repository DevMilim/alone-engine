use std::{any::Any, collections::VecDeque, time::Instant};

use assets::Resources;
use core::{Base, GameObjectDispatch, GlobalEvent, Id, Transform2D, Vector2};
use indexmap::IndexMap;
use input::{InputState, KeyCode};
use render::DrawCommand;

use crate::{context::EngineContext, scene::Scene};

const FIXED_DT: f32 = 1.0 / 60.0;
const MAX_ACCUM: f32 = 0.5;

#[derive(Debug, Clone)]
pub enum EngineCommands {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    MousePosition(f32, f32),
    Quit,
}

pub struct Engine<S: Scene> {
    objects: Vec<S>,
    base: Base,
    is_running: bool,
    pub input: InputState,
    pub event_queue: VecDeque<EngineCommands>,
    events: VecDeque<GlobalEvent>,
    mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,
    camera_pos: Vector2,
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
        }
    }
    pub fn update_step(&mut self) -> bool {
        self.input.clear_frame_data();

        self.process_commands();

        let now = Instant::now();
        let mut delta_time = (now - self.last_instant).as_secs_f32();
        self.last_instant = now;

        if delta_time >= MAX_ACCUM {
            delta_time = MAX_ACCUM;
        }

        self.accumulator += delta_time;
        self.update(delta_time);

        self.flush_messages_and_events();
        self.is_running
    }
    pub fn push(&mut self, scene: S) {
        self.objects.push(scene);
    }
    pub fn pop(&mut self) {
        let mut binding = Vec::new();
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut binding,
        );
        if let Some(mut scene) = self.objects.pop() {
            scene.get_dispatch().dispatch_destroy(&mut ctx);
        }
    }
    pub fn set_scene(&mut self, scene: S) {
        let mut binding = Vec::new();
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut binding,
        );
        while let Some(mut old_scene) = self.objects.pop() {
            old_scene.get_dispatch().dispatch_destroy(&mut ctx);
        }
        self.objects.clear();
        self.objects.push(scene);
    }

    pub fn update(&mut self, delta_time: f32) {
        let mut binding = Vec::new();
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut binding,
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
    }
    pub fn draw(&mut self) -> Vec<DrawCommand> {
        let mut render_queue = Vec::new();
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut render_queue,
        );
        if let Some(obj) = self.objects.last_mut() {
            obj.get_dispatch().dispatch_draw(&mut ctx, &self.base);
        } else {
            self.quit();
        }
        render_queue
    }
    pub fn flush_messages_and_events(&mut self) {
        let mut binding = Vec::new();
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
            &mut binding,
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
    pub fn process_commands(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            match event {
                EngineCommands::Quit => self.is_running = false,
                EngineCommands::KeyDown(keycode) => self.input.update_key(keycode, true),
                EngineCommands::KeyUp(keycode) => self.input.update_key(keycode, false),
                EngineCommands::MousePosition(x, y) => self.input.set_mouse_position(x, y),
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
