use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use rustc_hash::FxHashMap;

use crate::{
    collision::{AABB, ColliderData, ColliderKey},
    core::{
        AssetApi, Base, Component, EngineApi, GameObjectBase, Handler, Id, LdtkError, LdtkProject,
        LdtkTile, RenderApi, TilesetDef,
    },
    math::{Rect, Vector2, Vector2i},
    render::{Anchor, ImageAsset},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TileCollision {
    Full,
    Custom(Rect),
    OnWay,
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
    pub debug: bool,
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
    pub colliders: Vec<(Id, TileCollision, AABB)>,
    pub collision_rules: FxHashMap<i32, TileCollision>,
    pub collision_layer: u32,
}

impl Component for Tilemap {
    fn draw(&mut self, renderer: &mut impl RenderApi, base: &Base, _blending: f32) {
        if !self.visible {
            return;
        }
        for tile in &self.tiles {
            renderer.draw_sprite(
                base.position() + self.position + tile.position,
                tile.texture,
                Anchor::TopLeft,
                Some(tile.source),
                tile.flip_v,
                tile.flip_h,
                self.z_index,
            );
        }
    }
    fn update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, _delta: f32) {
        let origin = Vector2i::from(base.position() + self.position);
        for (key, collision_type, aabb) in &self.colliders {
            let is_on_way = *collision_type == TileCollision::OnWay;
            let data = ColliderData {
                aabb: AABB {
                    x: aabb.x + origin.x,
                    y: aabb.y + origin.y,
                    width: aabb.width,
                    height: aabb.height,
                },
                layer: self.collision_layer,
                mask: self.collision_layer,
                on_way_collision: is_on_way,
                is_sensor: false,
            };
            ctx.update_collider(
                ColliderKey {
                    key: *key,
                    id: base.id,
                },
                data,
            );
        }
    }
    fn destroy(&mut self, ctx: &mut impl EngineApi, base: &Base) {
        for (key, _, _) in &self.colliders {
            ctx.remove_collider(ColliderKey {
                key: *key,
                id: base.id,
            });
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
            collision_rules: FxHashMap::default(),
            collision_layer: 1,
        }
    }
    pub fn set_int_grid_rules(&mut self, rules: &[(i32, TileCollision)]) {
        for rule in rules {
            self.collision_rules.insert(rule.0, rule.1.clone());
        }
    }

    /// importa o IntGrid do LDtk e gera os colisores utilizando merge
    fn create_colliders(
        &mut self,
        int_grid_csv: &[i32],
        c_wid: u32,
        grid_size: i32,
        px_offset_x: i32,
        px_offset_y: i32,
        int_grid_rules: &[(i32, TileCollision)],
    ) {
        self.set_int_grid_rules(int_grid_rules);

        let width = c_wid as usize;
        let height = int_grid_csv.len() / width;

        let mut visited = vec![false; int_grid_csv.len()];

        let mut colliders: Vec<(AABB, TileCollision)> = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;

                if visited[index] {
                    continue;
                }

                let value = int_grid_csv[index];

                let collision = match self.collision_rules.get(&value) {
                    Some(c) => c.clone(),
                    None => continue,
                };

                if matches!(collision, TileCollision::None) {
                    continue;
                }

                let world_x = px_offset_x + x as i32 * grid_size;
                let world_y = px_offset_y + y as i32 * grid_size;
                if let TileCollision::Custom(custom) = collision {
                    visited[index] = true;

                    colliders.push((
                        AABB {
                            x: world_x + custom.x,
                            y: world_y + custom.y,
                            width: custom.width,
                            height: custom.height,
                        },
                        collision,
                    ));
                    continue;
                }

                let mut merge_w = 0;
                while x + merge_w < width {
                    let i = y * width + (x + merge_w);
                    if visited[i] || int_grid_csv[i] != value {
                        break;
                    }
                    visited[i] = true;
                    merge_w += 1;
                }

                colliders.push((
                    AABB {
                        x: world_x,
                        y: world_y,
                        width: merge_w as i32 * grid_size,
                        height: grid_size,
                    },
                    collision,
                ));
            }
        }
        for (collider, collision_type) in colliders.into_iter() {
            self.colliders.push((Id::new(), collision_type, collider))
        }
    }

    /// Carrega um arquivo do ltdk e gera o tilemap
    /// api recebe o ctx para acessar a engine
    /// json_path e onde o json exportado pelo ltdk esta
    /// level_key e o nome do level escolhido no ltdk
    pub fn from_ldtk_file<A: AssetApi, P: AsRef<Path>>(
        owner: Id,
        api: &mut A,
        json_path: P,
        level_key: &str,
        int_grid_rules: &[(i32, TileCollision)],
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
                "IntGrid" => {
                    if layer.identifier == "Collision" {
                        map.create_colliders(
                            &layer.int_grid_csv,
                            layer.c_wid,
                            layer.grid_size as i32,
                            layer.px_offset_x,
                            layer.px_offset_y,
                            int_grid_rules,
                        );
                    }
                    continue;
                }
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
                *existing
            } else {
                let tileset_path = base_dir.join(&tileset_def.rel_path);
                let texture = api.load_texture(owner, &tileset_path.to_string_lossy());
                tileset_cache.insert(tileset_uid, texture);
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
                    x: tile.src[0],
                    y: tile.src[1],
                    width: tileset_def.tile_grid_size as i32,
                    height: tileset_def.tile_grid_size as i32,
                };

                let position = Vector2 {
                    x: level.world_x as f32 + layer.px_offset_x as f32 + tile.px[0] as f32,
                    y: level.world_y as f32 + layer.px_offset_y as f32 + tile.px[1] as f32,
                };

                map.tiles.push(Tile {
                    texture,
                    source,
                    position,
                    flip_h: tile.f & 1 != 0,
                    flip_v: tile.f & 2 != 0,
                    tile_collision: TileCollision::None,
                    debug: false,
                });
            }
        }

        Ok(map)
    }
}
