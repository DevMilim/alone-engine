use std::{any::Any, collections::VecDeque, time::Instant};

use assets::Resources;
use core::{Base, DrawCommand, GlobalEvent, Id, Transform2D, Vector2};
use indexmap::IndexMap;
use input::InputState;
use render::{RenderCommands, Runtime, run_application};

use crate::{GameObjectDispatch, context::EngineContext, scene::Scene};

const FIXED_DT: f32 = 1.0 / 60.0;
const MAX_ACCUM: f32 = 0.5;

pub struct Engine<S: Scene> {
    objects: Vec<S>,
    base: Base,
    is_running: bool,
    pub input: InputState,
    pub event_queue: VecDeque<RenderCommands>,
    events: VecDeque<GlobalEvent>,
    mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,
    camera_pos: Vector2,
    pub resources: Resources,
    last_instant: Instant,
    accumulator: f32,
}

impl<S: Scene> Runtime for Engine<S> {
    fn update(&mut self) -> bool {
        self.update_step()
    }

    fn events(&mut self, cmd: render::RenderCommands) {
        match cmd {
            RenderCommands::Quit => self.is_running = false,
            RenderCommands::KeyDown(keycode) => self.input.update_key(keycode, true),
            RenderCommands::KeyUp(keycode) => self.input.update_key(keycode, false),
            RenderCommands::MousePosition(x, y) => self.input.set_mouse_position(x, y),
        }
    }

    fn render(&mut self, renderer: &mut impl core::RenderApi) {
        if let Some(obj) = self.objects.last_mut() {
            obj.get_dispatch().dispatch_draw(renderer, &self.base);
        } else {
            self.quit();
        }
    }

    fn get_texture(&self, handler: core::TextureHandler) -> Option<&assets::ImageAsset> {
        self.resources.textures.get(handler.id())
    }
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

    pub fn run(self) {
        run_application(self);
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
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
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
        );
        while let Some(mut old_scene) = self.objects.pop() {
            old_scene.get_dispatch().dispatch_destroy(&mut ctx);
        }
        self.objects.clear();
        self.objects.push(scene);
    }

    pub fn update(&mut self, delta_time: f32) {
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
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
    pub fn flush_messages_and_events(&mut self) {
        let mut ctx = EngineContext::new(
            &self.input,
            &mut self.event_queue,
            &mut self.events,
            &mut self.mailbox,
            &mut self.camera_pos,
            &mut self.resources,
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
        while let Some(event) = self.event_queue.pop_front() {}
    }
    pub fn quit(&mut self) {
        self.is_running = false;
    }

    pub fn clear_unecessary(&mut self) {
        self.resources.clear();
    }
}
