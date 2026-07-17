use std::fmt::Debug;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    audio::AudioAsset,
    core::{Handler, Id},
    render::ImageAsset,
};

/// Utilizado para armazenar assets em cache
#[derive(Debug)]
pub struct AssetCache<T: Debug> {
    assets: FxHashMap<usize, T>,
    path_map: FxHashMap<String, usize>,
    next_id: usize,
    asset_users: FxHashMap<usize, FxHashSet<Id>>,

    object_assets: FxHashMap<Id, FxHashSet<usize>>,
}

impl<T: Debug> AssetCache<T> {
    pub fn new() -> Self {
        Self {
            assets: FxHashMap::default(),
            path_map: FxHashMap::default(),
            next_id: 0,
            asset_users: FxHashMap::default(),
            object_assets: FxHashMap::default(),
        }
    }
    /// Retorna um id para uso
    fn current_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
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
    pub fn insert(&mut self, game_object_id: Id, path: &str, asset: T) -> Handler<T> {
        if let Some(asset_id) = self.path_map.get(path).copied() {
            self.asset_users
                .entry(asset_id)
                .or_default()
                .insert(game_object_id);

            self.object_assets
                .entry(game_object_id)
                .or_default()
                .insert(asset_id);
            return Handler::new(asset_id);
        }
        let asset_id = self.current_id();
        self.assets.insert(asset_id, asset);
        self.path_map.insert(path.to_string(), asset_id);
        self.asset_users
            .entry(asset_id)
            .or_default()
            .insert(game_object_id);

        self.object_assets
            .entry(game_object_id)
            .or_default()
            .insert(asset_id);
        Handler::new(asset_id)
    }
    pub fn clear(&mut self) {
        self.assets.clear();
        self.path_map.clear();
        self.object_assets.clear();
        self.asset_users.clear();
    }
    pub fn remove(&mut self, game_object_id: Id, asset_id: Handler<T>) {
        let id = asset_id.id;

        let mut object_empty = false;
        if let Some(assets) = self.object_assets.get_mut(&game_object_id) {
            assets.remove(&id);
            object_empty = assets.is_empty();
        }
        if object_empty {
            self.object_assets.remove(&game_object_id);
        }

        let should_remove = if let Some(users) = self.asset_users.get_mut(&id) {
            users.remove(&game_object_id);
            users.is_empty()
        } else {
            false
        };

        if should_remove {
            self.asset_users.remove(&id);
            self.assets.remove(&id);
            self.path_map.retain(|_, v| *v != id);
        }
    }
    pub fn remove_game_object(&mut self, game_object: Id) {
        if let Some(assets) = self.object_assets.remove(&game_object) {
            for asset_id in assets {
                let should_remove = if let Some(users) = self.asset_users.get_mut(&asset_id) {
                    users.remove(&game_object);
                    users.is_empty()
                } else {
                    false
                };

                if should_remove {
                    self.asset_users.remove(&asset_id);
                    self.assets.remove(&asset_id);
                    self.path_map.retain(|_, id| *id != asset_id);
                }
            }
        }
    }
}

impl<T: Debug> Default for AssetCache<T> {
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
