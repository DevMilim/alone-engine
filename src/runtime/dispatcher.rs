use crate::{Base, EngineApi, GameObject, GlobalEvent, RenderApi};

pub trait GameObjectDispatch {
    fn is_pending_removal(&self) -> bool {
        false
    }
    fn dispatch_start(&mut self, ctx: &mut impl EngineApi, base: &Base);
    fn dispatch_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32);
    fn dispatch_late_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32);
    fn dispatch_fixed_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32);
    fn dispatch_event(&mut self, ctx: &mut impl EngineApi, event: &GlobalEvent);
    fn dispatch_message(&mut self, ctx: &mut impl EngineApi);
    fn dispatch_draw(&mut self, ctx: &mut impl RenderApi, base: &Base, blending: f32);
    fn dispatch_destroy(&mut self, ctx: &mut impl EngineApi);
}

impl<T: GameObjectDispatch + GameObject> GameObjectDispatch for Vec<T> {
    fn dispatch_start(&mut self, ctx: &mut impl EngineApi, base: &Base) {
        self.retain_mut(|obj| {
            obj.dispatch_start(ctx, base);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32) {
        self.retain_mut(|obj| {
            obj.dispatch_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_late_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32) {
        self.retain_mut(|obj| {
            obj.dispatch_late_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_fixed_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32) {
        self.retain_mut(|obj| {
            obj.dispatch_fixed_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_draw(&mut self, ctx: &mut impl RenderApi, base: &Base, blending: f32) {
        for obj in self.iter_mut() {
            obj.dispatch_draw(ctx, base, blending);
        }
    }

    fn dispatch_destroy(&mut self, ctx: &mut impl EngineApi) {
        for obj in self.iter_mut() {
            obj.dispatch_destroy(ctx);
        }
    }

    fn dispatch_event(&mut self, ctx: &mut impl EngineApi, event: &GlobalEvent) {
        for obj in self.iter_mut() {
            obj.dispatch_event(ctx, event);
        }
    }

    fn dispatch_message(&mut self, ctx: &mut impl EngineApi) {
        for obj in self.iter_mut() {
            obj.dispatch_message(ctx);
        }
    }
}

impl<T: GameObjectDispatch + GameObject> GameObjectDispatch for Option<T> {
    fn is_pending_removal(&self) -> bool {
        match self {
            Some(obj) => obj.is_pending_removal(),
            None => false,
        }
    }
    fn dispatch_start(&mut self, ctx: &mut impl EngineApi, base: &Base) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_start(ctx, base);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_late_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_late_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_fixed_update(&mut self, ctx: &mut impl EngineApi, base: &Base, delta: f32) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_fixed_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_draw(&mut self, ctx: &mut impl RenderApi, base: &Base, blending: f32) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_draw(ctx, base, blending);
        }
    }

    fn dispatch_destroy(&mut self, ctx: &mut impl EngineApi) {
        if let Some(obj) = self.as_mut() {
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_event(&mut self, ctx: &mut impl EngineApi, event: &GlobalEvent) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_event(ctx, event);
        }
    }

    fn dispatch_message(&mut self, ctx: &mut impl EngineApi) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_message(ctx);
        }
    }
}
