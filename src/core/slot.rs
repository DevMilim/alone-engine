use crate::core::{Base, Component, EMPTY_BASE, EngineApi, GameObject, GameObjectBase, RenderApi};

pub struct Slot<T> {
    pub(crate) inner: Option<T>,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self { inner: None }
    }
}

impl<T> Slot<T> {
    pub fn new(inner: T) -> Self {
        Self { inner: Some(inner) }
    }
    pub fn unwrap(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }
}

impl<T: GameObject> Slot<T> {
    pub fn remove(&mut self) {
        if let Some(inner) = &mut self.inner {
            inner.base_mut().queue_free();
        }
    }
}

impl<T: GameObjectBase> GameObjectBase for Slot<T> {
    fn base(&self) -> &Base {
        match &self.inner {
            Some(obj) => obj.base(),
            None => &EMPTY_BASE,
        }
    }

    fn base_mut(&mut self) -> &mut Base {
        match &mut self.inner {
            Some(obj) => obj.base_mut(),
            None => panic!("Tentativa de acessar base_mut em um Option vazio."),
        }
    }
}

impl<T: Component> Component for Slot<T> {
    fn start(&mut self, ctx: &mut impl EngineApi, base: &mut Base) {
        if let Some(component) = &mut self.inner {
            component.start(ctx, base);
        }
    }
    fn update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if let Some(component) = &mut self.inner {
            component.update(ctx, base, delta);
        }
    }
    fn late_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if let Some(component) = &mut self.inner {
            component.late_update(ctx, base, delta);
        }
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if let Some(component) = &mut self.inner {
            component.fixed_update(ctx, base, delta);
        }
    }
    fn draw(&mut self, renderer: &mut impl RenderApi, base: &Base, blending: f32) {
        if let Some(component) = &mut self.inner {
            component.draw(renderer, base, blending);
        }
    }
    fn destroy(&mut self, ctx: &mut impl EngineApi, base: &Base) {
        if let Some(component) = &mut self.inner {
            component.destroy(ctx, base);
        }
    }
}

impl<T: GameObject> GameObject for Slot<T> {
    type Message = ();
}
