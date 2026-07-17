use std::time::Instant;

use crate::{
    core::Base,
    render::{DrawCommand, RenderQueue},
    runtime::{EngineContext, GameObjectDispatch, Scene},
};

pub const FIXED_DT: f32 = 1.0 / 60.0;
pub const MAX_ACCUM: f32 = 0.5;

pub enum SceneError {}

pub struct WorldState<S: Scene, P: GameObjectDispatch> {
    pub scenes: Vec<S>,
    pub global: Option<P>,
    pub accumulator: f32,
    pub last_instant: Instant,
    pub is_running: bool,
}

impl<S: Scene, P: GameObjectDispatch> WorldState<S, P> {
    pub fn new(root_scene: S) -> Self {
        Self {
            scenes: vec![root_scene],
            accumulator: 0.0,
            last_instant: Instant::now(),
            global: None,
            is_running: true,
        }
    }
    pub fn change_scene(&mut self, scene: S, ctx: &mut EngineContext) {
        if let Some(mut old) = self.scenes.pop() {
            old.get_dispatch().dispatch_destroy(ctx);
        }
        self.scenes.push(scene);
    }
    pub fn pop_scene(&mut self, ctx: &mut EngineContext) {
        if self.scenes.len() <= 1 {
            eprintln!("Tentativa de pop_scene na ultima cena da pilha");
            return;
        }
        if let Some(mut scene) = self.scenes.pop() {
            scene.get_dispatch().dispatch_destroy(ctx);
        }
    }
    pub fn push_scene(&mut self, scene: S) {
        self.scenes.push(scene);
    }
    pub fn clear_scenes(&mut self, ctx: &mut EngineContext) {
        if self.scenes.len() <= 1 {
            return;
        }
        let keep = self.scenes.len() - 1;
        for mut scene in self.scenes.drain(..keep) {
            scene.get_dispatch().dispatch_destroy(ctx);
        }
    }
    pub fn last_scene(&mut self) -> &mut impl GameObjectDispatch {
        self.scenes
            .last_mut()
            .expect("WorldState.scenes não deveria ficar vazio")
            .get_dispatch()
    }
    pub fn render(
        &mut self,
        renderer: &mut [Vec<DrawCommand>; 6],
        ctx: &mut EngineContext,
        base: &Base,
        blending: f32,
    ) {
        let mut render_ctx = RenderQueue {
            queue: renderer,
            camera: ctx.camera_position,
        };
        if let Some(global) = &mut self.global {
            global.dispatch_draw(&mut render_ctx, base, blending);
        }
        if let Some(obj) = self.scenes.last_mut() {
            obj.get_dispatch()
                .dispatch_draw(&mut render_ctx, base, blending);
        } else {
            self.quit();
        }
    }

    pub fn update(
        &mut self,
        ctx: &mut EngineContext,
        base: &Base,
        fixed_update_count: &mut u64,
    ) -> (bool, f32) {
        let now = Instant::now();
        let mut delta_time = (now - self.last_instant).as_secs_f32();
        self.last_instant = now;

        if delta_time >= MAX_ACCUM {
            delta_time = MAX_ACCUM;
        }

        self.accumulator += delta_time;

        let Some(object) = self.scenes.last_mut() else {
            self.quit();
            return (false, 0.0);
        };

        let object = object.get_dispatch();

        if let Some(global) = &mut self.global {
            global.dispatch_update(ctx, base, delta_time);
        }
        object.dispatch_update(ctx, base, delta_time);

        while self.accumulator >= FIXED_DT {
            ctx.set_fixed_update(true);
            *fixed_update_count += 1;
            ctx.systems.input.current_fixed_frame = *fixed_update_count;
            ctx.systems.collision.rebuild_grid();

            if let Some(global) = &mut self.global {
                global.dispatch_fixed_update(ctx, base, FIXED_DT);
            }
            object.dispatch_fixed_update(ctx, base, FIXED_DT);

            let global_events = ctx.systems.collision_step();
            for event in global_events {
                ctx.events.global_events.push_back(event);
            }

            self.accumulator -= FIXED_DT;
            ctx.set_fixed_update(false);
        }

        if let Some(global) = &mut self.global {
            global.dispatch_late_update(ctx, base, delta_time);
        }
        object.dispatch_late_update(ctx, base, delta_time);

        let blending = self.accumulator / FIXED_DT;
        (self.is_running, blending)
    }
    pub fn quit(&mut self) {
        self.is_running = false
    }
}
