use rustc_hash::FxHashMap;

use crate::{AudioAsset, Handler, ImageAsset};

pub struct AssetCache<T> {
    assets: FxHashMap<usize, T>,
    path_map: FxHashMap<String, usize>,
    next_id: usize,
}

impl<T> AssetCache<T> {
    pub fn new() -> Self {
        Self {
            assets: FxHashMap::default(),
            path_map: FxHashMap::default(),
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
        if let Some(id) = self.path_map.get(path).copied() {
            return Handler::new(id);
        }
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

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
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
