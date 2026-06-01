use rustc_hash::FxHashMap;

use crate::{AudioAsset, Handler, ImageAsset};

/// Utilizado para armazenar assets em cache
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
    /// Retorna um id para uso
    fn current_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }
    /// Obtem o id de um asset pelo seu caminho
    pub fn get_id(&self, path: &str) -> Option<usize> {
        self.path_map.get(path).copied()
    }
    /// Retorna um asset usando handler
    pub fn get(&self, id: Handler<T>) -> Option<&T> {
        self.assets.get(&id.id)
    }
    /// Obtem um asset mutavel
    pub fn get_mut(&mut self, id: Handler<T>) -> Option<&mut T> {
        self.assets.get_mut(&id.id)
    }
    /// insere um asset
    pub fn insert(&mut self, path: &str, asset: T) -> Handler<T> {
        if let Some(id) = self.path_map.get(path).copied() {
            return Handler::new(id);
        }
        let id = self.current_id();
        self.assets.insert(id, asset);
        self.path_map.insert(path.to_string(), id);
        Handler::new(id)
    }
    /// Limpa todos os assets do cache
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

/// Utilizado para agrupar tipos diferentes de assets
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
    /// limpa todos os assets
    pub fn clear(&mut self) {
        self.textures.clear();
        self.sounds.clear();
    }
}
