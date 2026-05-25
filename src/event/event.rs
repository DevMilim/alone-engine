use std::cell::RefCell;

use crate::{ColliderKey, GameObject, Id};

#[derive(Debug, Clone)]
pub enum TriggerKind {
    Enter,
    Exit,
}
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub owner: Id,
    pub sensor: ColliderKey,
    pub kind: TriggerKind,
}
pub struct SpawnEvent<T> {
    payload: RefCell<Option<T>>,
}

impl<T: GameObject + GameObject> SpawnEvent<T> {
    pub fn new(obj: T) -> Self {
        Self {
            payload: RefCell::new(Some(obj)),
        }
    }
    pub fn take(&self) -> Option<T> {
        self.payload.borrow_mut().take()
    }
}
