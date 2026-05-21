use std::cell::RefCell;

use core::GameObject;

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
