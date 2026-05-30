use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use crate::{
    AABB, Anchor, AssetApi, Base, Component, GameObjectBase, Handler, ImageAsset, LdtkError,
    LdtkProject, LdtkTile, Rect, TilesetDef, Vector2,
};

#[derive(Debug, Clone)]
pub enum TileCollision {
    Full,
    Custom(AABB),
    None,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub texture: Handler<ImageAsset>,
    pub source: Rect,
    pub position: Vector2,
    pub flip_h: bool,
    pub flip_v: bool,
    pub tile_collision: TileCollision,
}

#[derive(Debug, Clone)]
pub struct Tilemap {
    pub tiles: Vec<Tile>,
    pub x_cells: u32,
    pub y_cells: u32,

    pub tile_size: Vector2,
    pub visible: bool,
    pub previous_position: Vector2,
    pub position: Vector2,
    pub z_index: u8,
    pub colliders: Vec<AABB>,
}

impl Component for Tilemap {
    fn draw(&mut self, renderer: &mut impl crate::RenderApi, base: &Base, _blending: f32) {
        if !self.visible {
            return;
        }
        for tile in &self.tiles {
            renderer.draw_sprite(
                base.position() + self.position + tile.position,
                tile.texture,
                Anchor::TopLeft,
                Some(tile.source),
                self.z_index,
            );
        }
    }
}

impl Tilemap {
    pub fn empty(tile_size: Vector2) -> Self {
        Self {
            tiles: Vec::new(),
            x_cells: 0,
            y_cells: 0,
            tile_size,
            visible: true,
            previous_position: Vector2 { x: 0.0, y: 0.0 },
            position: Vector2 { x: 0.0, y: 0.0 },
            z_index: 0,
            colliders: Vec::new(),
        }
    }
    pub fn import_int_grid(&mut self, _grid: &[(i32, TileCollision)]) {}
    /// Carrega um arquivo do ltdk e gera o tilemap
    /// api recebe o ctx para acessar a engine
    /// json_path e onde o json exportado pelo ltdk esta
    /// level_key e o nome do level escolhido no ltdk
    pub fn from_ldtk_file<A: AssetApi, P: AsRef<Path>>(
        api: &mut A,
        json_path: P,
        level_key: &str,
    ) -> Result<Self, LdtkError> {
        let json_path = json_path.as_ref();
        let json = File::open(json_path)?;
        let reader = BufReader::new(json);
        let project: LdtkProject = serde_json::from_reader(reader)?;

        let level = project
            .levels
            .iter()
            .find(|level| level.iid == level_key || level.identifier == level_key)
            .ok_or_else(|| LdtkError::LevelNotFound(level_key.to_string()))?;

        let base_dir = json_path.parent().unwrap_or(Path::new("."));

        let tileset_map: HashMap<i64, &TilesetDef> = project
            .defs
            .tilesets
            .iter()
            .map(|ts| (ts.uid, ts))
            .collect();

        let mut tileset_cache: HashMap<i64, Handler<ImageAsset>> = HashMap::new();

        let mut map = Tilemap::empty(Vector2 { x: 16.0, y: 16.0 });

        for layer in &level.layer_instances {
            let layer_tiles: &[LdtkTile] = match layer.layer_type.as_str() {
                "Tiles" => &layer.grid_tiles,
                "AutoLayer" => &layer.auto_tiles,
                _ => continue,
            };

            let tileset_uid = layer
                .override_tileset_uid
                .or(layer.tileset_def_uid)
                .ok_or_else(|| LdtkError::MissingTilesetForLayer(layer.identifier.clone()))?;

            let tileset_def = tileset_map
                .get(&tileset_uid)
                .ok_or(LdtkError::TilesetDefNotFound(tileset_uid))?;

            let texture = if let Some(existing) = tileset_cache.get(&tileset_uid) {
                existing.clone()
            } else {
                let tileset_path = base_dir.join(&tileset_def.rel_path);
                let texture = api.load_texture(&tileset_path.to_string_lossy());
                tileset_cache.insert(tileset_uid, texture.clone());
                texture
            };

            map.tile_size = Vector2 {
                x: tileset_def.tile_grid_size as f32,
                y: tileset_def.tile_grid_size as f32,
            };

            map.x_cells = map.x_cells.max(layer.c_wid);
            map.y_cells = map.y_cells.max(layer.c_hei);

            for tile in layer_tiles {
                let source = Rect {
                    x: tile.src[0] as f32,
                    y: tile.src[1] as f32,
                    width: tileset_def.tile_grid_size,
                    height: tileset_def.tile_grid_size,
                };

                let position = Vector2 {
                    x: level.world_x as f32 + layer.px_offset_x as f32 + tile.px[0] as f32,
                    y: level.world_y as f32 + layer.px_offset_y as f32 + tile.px[1] as f32,
                };

                map.tiles.push(Tile {
                    texture: texture.clone(),
                    source,
                    position,
                    flip_h: tile.f & 1 != 0,
                    flip_v: tile.f & 2 != 0,
                    tile_collision: TileCollision::None,
                });
            }
        }

        Ok(map)
    }
}
