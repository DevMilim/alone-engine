use std::{any::Any, cell::RefCell};

use engine_core::{GameObject, GameObjectDispatch, Id};

pub struct SpawnEvent<T> {
    payload: RefCell<Option<T>>,
}

impl<T: GameObject + GameObjectDispatch> SpawnEvent<T> {
    pub fn new(obj: T) -> Self {
        Self {
            payload: RefCell::new(Some(obj)),
        }
    }
    pub fn take(&self) -> Option<T> {
        self.payload.borrow_mut().take()
    }
}
