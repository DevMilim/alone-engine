use std::{collections::HashMap, marker::PhantomData};

pub struct AssetCache<T> {
    assets: HashMap<usize, T>,
    path_map: HashMap<String, usize>,
    next_id: usize,
}

impl<T> AssetCache<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_map: HashMap::new(),
            next_id: 0,
        }
    }
    fn current_id(&mut self) -> Handler<T> {
        self.next_id += 1;
        Handler::new(self.next_id)
    }
    pub fn get_id(&self, path: &str) -> Option<usize> {
        self.path_map.get(path).copied()
    }
    pub fn get(&self, id: Handler<T>) -> Option<&T> {
        self.assets.get(&id.id)
    }
    pub fn get_mut(&mut self, id: Handler<T>) -> Option<&mut T> {
        self.assets.get_mut(&id.id)
    }
    pub fn insert(&mut self, path: &str, asset: T) -> Handler<T> {
        let id = self.current_id();
        self.assets.insert(id.id, asset);
        self.path_map.insert(path.to_string(), id.id);
        id
    }
    pub fn clear(&mut self) {
        self.assets.clear();
        self.path_map.clear();
        self.next_id = 0;
    }
}

impl<T> Default for AssetCache<T> {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Handler<T> {
    pub id: usize,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Handler<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T> Copy for Handler<T> {}

impl<T> Handler<T> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

pub struct Resources {}

impl Resources {
    pub fn clear(&mut self) {}
}
