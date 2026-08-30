use std::any::{Any, TypeId};

use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct State {
    data: FxHashMap<TypeId, Box<dyn Any>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            data: FxHashMap::default(),
        }
    }
    pub fn set<T: 'static>(&mut self, val: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(val));
    }
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut())
    }
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.data
            .remove(&TypeId::of::<T>())
            .and_then(|value| value.downcast::<T>().ok())
            .map(|value| *value)
    }
    pub fn contains<T: 'static>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }
}
