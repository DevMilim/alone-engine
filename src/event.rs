use std::{any::Any, cell::RefCell};

use crate::{GameObject, GameObjectDispatch, Id};

pub enum GlobalEvent {
    Broadcast(Box<dyn Any>),
    Targeted(Id, Box<dyn Any>),
}

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
