use crate::{Handler, ImageAsset, Vector2};

pub struct Tile {
    texture: Handler<ImageAsset>,
    x: u32,
    y: u32,
}

pub struct Tilemap {
    pub tiles: Vec<Tile>,
    x_cells: u32,
    y_cells: u32,

    pub tile_size: Vector2,
    pub visible: bool,
    pub previous_position: Vector2,
}
