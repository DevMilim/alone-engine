use crate::{Id, Transform2D, Vector2};

#[derive(Clone)]
pub struct Base {
    pub id: Id,
    pub transform: Transform2D,
    pub pending_removal: bool,
    pub top_level: bool,
    pub z_index: i32,
}

impl Base {
    pub fn new(transform: Transform2D) -> Self {
        Self {
            transform,
            pending_removal: false,
            top_level: false,
            id: Id::new(),
            z_index: 0,
        }
    }
    pub fn queue_free(&mut self) {
        self.pending_removal = true;
    }
}

pub trait GameObjectBase {
    fn base(&self) -> &Base;
    fn base_mut(&mut self) -> &mut Base;

    fn position(&self) -> Vector2 {
        self.base().transform.position
    }
    fn rotation(&self) -> f32 {
        self.base().transform.rotation
    }
    fn scale(&self) -> Vector2 {
        self.base().transform.scale
    }

    fn global_position(&self) -> Vector2 {
        self.base().transform.global_position
    }
    fn global_rotation(&self) -> f32 {
        self.base().transform.global_rotation
    }
    fn global_scale(&self) -> Vector2 {
        self.base().transform.global_scale
    }

    fn set_position(&mut self, position: Vector2) {
        self.base_mut().transform.position = position;
    }
    fn set_rotation(&mut self, rotation: f32) {
        self.base_mut().transform.rotation = rotation;
    }
    fn set_scale(&mut self, scale: Vector2) {
        self.base_mut().transform.scale = scale;
    }

    fn set_global_position(&mut self, position: Vector2) {
        self.base_mut().transform.global_position = position;
    }
    fn set_global_rotation(&mut self, rotation: f32) {
        self.base_mut().transform.global_rotation = rotation;
    }
    fn set_global_scale(&mut self, scale: Vector2) {
        self.base_mut().transform.global_scale = scale;
    }

    fn queue_free(&mut self) {
        self.base_mut().pending_removal = true;
    }
    fn z_index(&self) -> i32 {
        self.base().z_index
    }
    fn set_z_index(&mut self, z_index: i32) {
        self.base_mut().z_index = z_index;
    }
}
