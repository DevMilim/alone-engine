use std::any::Any;

use indexmap::IndexMap;
use rodio::Player;
use winit::keyboard::KeyCode;

use crate::{
    Anchor, AudioAsset, ColliderData, ColliderKey, Color, DrawCommand, GameObject, Handler, Id,
    ImageAsset, Rect, Vector2,
};

pub trait EngineApi: InputApi + AssetApi + EventApi + AudioApi + CollisionApi {
    fn quit(&mut self);
    fn mailbox(&mut self) -> &mut IndexMap<Id, Vec<Box<dyn Any>>>;
    fn camera_mut(&mut self) -> &mut Vector2;
}
pub trait InputApi {
    fn is_key_pressed(&self, key: KeyCode) -> bool;
    fn is_key_just_pressed(&self, key: KeyCode) -> bool;
    fn mouse_position(&self) -> Vector2;
    fn is_action_pressed(&self, action: &str) -> bool;
    fn is_action_just_pressed(&self, action: &str) -> bool;
    fn get_vector(
        &self,
        action_up: &str,
        action_down: &str,
        action_left: &str,
        action_right: &str,
    ) -> Vector2;
    fn get_key_vector(
        &self,
        key_up: KeyCode,
        key_down: KeyCode,
        key_left: KeyCode,
        key_right: KeyCode,
    ) -> Vector2;
}

pub trait AssetApi {
    fn load_texture(&mut self, path: &str) -> Handler<ImageAsset>;
    fn load_texture_and_resize(
        &mut self,
        path: &str,
        width: u32,
        height: u32,
    ) -> Handler<ImageAsset>;
}
pub trait EventApi {
    fn send<T: 'static>(&mut self, id: Id, message: T);
    fn send_boxed_any(&mut self, id: Id, message: Box<dyn Any + 'static>);
    fn emit<T: 'static>(&mut self, event: T);
    fn emit_targeted<T: 'static>(&mut self, id: Id, event: T);
    fn spawn<T: GameObject + 'static>(&mut self, obj: T);
    fn mail_box_is_empty(&self) -> bool;
}
pub trait CollisionApi {
    fn update_collider(&mut self, key: ColliderKey, data: ColliderData);
    fn remove_collider(&mut self, key: ColliderKey);
    fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
        delta: f32,
    );
}

pub trait RenderApi {
    fn draw(&mut self, z_index: u8, command: DrawCommand);
    fn draw_rect(&mut self, rect: Rect, color: Color, z_index: u8);
    fn draw_sprite(
        &mut self,
        position: Vector2,
        texture: Handler<ImageAsset>,
        anchor: Anchor,
        z_index: u8,
    );
    fn camera_mut(&mut self) -> &mut Vector2;
}
pub trait AudioApi {
    fn play(&mut self, sound: Handler<AudioAsset>) -> Player;
    fn load_audio(&mut self, path: &str) -> Handler<AudioAsset>;
}
