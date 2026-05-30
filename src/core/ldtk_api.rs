use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct LdtkProject {
    pub defs: LdtkDefs,

    #[serde(default)]
    pub levels: Vec<LdtkLevel>,
}

#[derive(Debug, Deserialize)]
pub struct LdtkDefs {
    #[serde(default)]
    pub tilesets: Vec<TilesetDef>,
}

#[derive(Debug, Deserialize)]
pub struct TilesetDef {
    pub uid: i64,

    #[serde(rename = "relPath")]
    pub rel_path: String,

    #[serde(rename = "pxWid")]
    pub px_wid: u32,

    #[serde(rename = "pxHei")]
    pub px_hei: u32,

    #[serde(rename = "tileGridSize")]
    pub tile_grid_size: u32,

    #[serde(default)]
    pub spacing: u32,

    #[serde(default)]
    pub padding: u32,
}

#[derive(Debug, Deserialize)]
pub struct LdtkLevel {
    pub iid: String,
    pub identifier: String,

    #[serde(rename = "worldX")]
    pub world_x: i32,

    #[serde(rename = "worldY")]
    pub world_y: i32,

    #[serde(rename = "pxWid")]
    pub px_wid: u32,

    #[serde(rename = "pxHei")]
    pub px_hei: u32,

    #[serde(rename = "layerInstances", default)]
    pub layer_instances: Vec<LayerInstance>,
}

#[derive(Debug, Deserialize)]
pub struct LayerInstance {
    #[serde(rename = "__identifier")]
    pub identifier: String,

    #[serde(rename = "__type")]
    pub layer_type: String,

    #[serde(rename = "__cWid")]
    pub c_wid: u32,

    #[serde(rename = "__cHei")]
    pub c_hei: u32,

    #[serde(rename = "__gridSize")]
    pub grid_size: u32,

    #[serde(rename = "layerDefUid")]
    pub layer_def_uid: i64,

    #[serde(rename = "pxOffsetX", default)]
    pub px_offset_x: i32,

    #[serde(rename = "pxOffsetY", default)]
    pub px_offset_y: i32,

    #[serde(rename = "__tilesetDefUid", default)]
    pub tileset_def_uid: Option<i64>,

    #[serde(rename = "overrideTilesetUid", default)]
    pub override_tileset_uid: Option<i64>,

    #[serde(rename = "gridTiles", default)]
    pub grid_tiles: Vec<LdtkTile>,

    #[serde(rename = "autoTiles", default)]
    pub auto_tiles: Vec<LdtkTile>,
}

#[derive(Debug, Deserialize)]
pub struct LdtkTile {
    pub px: [i32; 2],
    pub src: [i32; 2],

    #[serde(default)]
    pub f: u8,
}

#[derive(Debug, Error)]
pub enum LdtkError {
    #[error("Erro de IO {0}")]
    Io(std::io::Error),
    #[error("Erro ao parsear LDtk JSON {0}")]
    Json(serde_json::Error),
    #[error("level não encontrado {0}")]
    LevelNotFound(String),
    #[error("tileset def não encontrado para uid {0}")]
    TilesetDefNotFound(i64),
    #[error("layer sem tileset associado {0}")]
    MissingTilesetForLayer(String),
}

impl From<std::io::Error> for LdtkError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for LdtkError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
