use std::collections::HashMap;

use crate::{AudioAsset, Handler, ImageAsset};

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
    fn current_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
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
        self.assets.insert(id, asset);
        self.path_map.insert(path.to_string(), id);
        Handler::new(id)
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

pub struct Resources {
    pub textures: AssetCache<ImageAsset>,
    pub sounds: AssetCache<AudioAsset>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            textures: AssetCache::new(),
            sounds: AssetCache::new(),
        }
    }
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}
