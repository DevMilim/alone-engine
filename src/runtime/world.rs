use std::time::Instant;

use crate::{Base, DrawCommand, EngineContext, GameObjectDispatch, RenderQueue, Scene};

pub const FIXED_DT: f32 = 1.0 / 60.0;
pub const MAX_ACCUM: f32 = 0.5;

pub struct WorldState<S: Scene> {
    pub objects: Vec<S>,
    pub accumulator: f32,
    pub last_instant: Instant,
    pub is_running: bool,
}

impl<S: Scene> WorldState<S> {
    pub fn new(root_scene: S) -> Self {
        Self {
            objects: vec![root_scene],
            accumulator: 0.0,
            last_instant: Instant::now(),
            is_running: true,
        }
    }
    pub fn last_scene(&mut self) -> &mut impl GameObjectDispatch {
        self.objects.last_mut().unwrap().get_dispatch()
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
        if let Some(obj) = self.objects.last_mut() {
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

        let Some(object) = self.objects.last_mut() else {
            panic!("Nenhum objeto iniciado")
        };

        let object = object.get_dispatch();

        object.dispatch_update(ctx, base, delta_time);

        while self.accumulator >= FIXED_DT {
            ctx.set_fixed_update(true);
            *fixed_update_count += 1;
            ctx.systems.input.current_fixed_frame = *fixed_update_count;
            object.dispatch_fixed_update(ctx, base, FIXED_DT);

            let global_events = ctx.systems.collision_step();
            for event in global_events {
                ctx.events.global_events.push_back(event);
            }

            self.accumulator -= FIXED_DT;
            ctx.set_fixed_update(false);
        }

        object.dispatch_late_update(ctx, base, delta_time);

        let blending = self.accumulator / FIXED_DT;
        (self.is_running, blending)
    }
    pub fn quit(&mut self) {
        self.is_running = false
    }
}
