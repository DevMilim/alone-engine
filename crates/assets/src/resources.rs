use std::{collections::HashMap, path::Path};

use core::Handler;

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

pub struct Resources {
    pub textures: AssetCache<ImageAsset>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            textures: AssetCache::new(),
        }
    }
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageAsset {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
impl ImageAsset {
    pub fn load_from_file(path: &str) -> Self {
        let img = image::open(Path::new(path)).expect("Falha ao carregar textura");
        let rgba = img.to_rgba8();

        let dimensions = rgba.dimensions();

        Self {
            width: dimensions.0,
            height: dimensions.1,
            pixels: rgba.to_vec(),
        }
    }
}
