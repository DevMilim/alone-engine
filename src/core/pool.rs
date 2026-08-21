use crate::{
    core::{Base, EMPTY_BASE, GameObject, GameObjectBase},
    runtime::GameObjectDispatch,
};

pub struct Pool<T: GameObject + GameObjectDispatch> {
    pub(crate) items: Vec<T>,
}

impl<T: GameObject + GameObjectDispatch> Default for Pool<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T: GameObject + GameObjectDispatch> Pool<T> {
    pub fn spawn(&mut self, object: T) {
        self.items.push(object);
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }
    pub fn queue_free_all(&mut self) {
        for item in &mut self.items {
            item.base_mut().queue_free();
        }
    }
}

impl<T: GameObject + GameObjectDispatch> GameObject for Pool<T> {
    type Message = ();
}
impl<T: GameObjectBase + GameObject + GameObjectDispatch> GameObjectBase for Pool<T> {
    fn base(&self) -> &Base {
        &EMPTY_BASE
    }

    fn base_mut(&mut self) -> &mut Base {
        panic!("Tentativa invalida de acessar base_mut em um Vec<GameObject>")
    }
}
