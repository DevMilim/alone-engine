use std::{any::Any, net::SocketAddr};

use bincode::{Decode, Encode};
use indexmap::IndexMap;
use rodio::Player;
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{
    audio::AudioAsset,
    collision::{ColliderData, ColliderKey, CollisionFlag},
    core::{GameObject, Handler, Id},
    math::{Color, Rect, Vector2},
    network::NetworkError,
    render::{Anchor, DrawCommand, ImageAsset},
    runtime::{AsyncContext, Scene},
};

pub trait EngineApi:
    CoreApi + WorldApi + InputApi + AssetApi + EventApi + AudioApi + CollisionApi + ServerApi + SceneApi
{
}

pub trait CoreApi {
    fn camera_mut(&mut self) -> &mut Vector2;
    fn window_size(&self) -> (u32, u32);
    fn async_ctx(&self) -> AsyncContext;
    fn async_task<F>(&mut self, owner_id: Id, future: F)
    where
        F: Future<Output = ()> + Send + 'static;
    fn abort_tasks_of(&mut self, id: Id);
}

pub trait WorldApi {
    fn spawn<T: GameObject + 'static>(&mut self, obj: T);
    fn register_alive(&mut self, id: Id);
    fn unregister_alive(&mut self, id: Id);
}

pub trait ServerApi {
    fn emit_to_server<E: Encode + Decode<()>>(&mut self, event: E) -> Result<(), NetworkError>;
    fn emit_to_client<E: Encode + Decode<()>>(
        &mut self,
        target: SocketAddr,
        event: E,
    ) -> Result<(), NetworkError>;
    fn send_to_server<E: Encode + Decode<()>>(
        &mut self,
        id: Id,
        message: E,
    ) -> Result<(), NetworkError>;
    fn send_to_client<E: Encode + Decode<()>>(
        &mut self,
        id: Id,
        target: SocketAddr,
        message: E,
    ) -> Result<(), NetworkError>;
}
pub trait InputApi {
    fn is_key_pressed(&self, key: KeyCode) -> bool;
    fn is_key_just_pressed(&self, key: KeyCode) -> bool;
    fn mouse_position(&self) -> Vector2;
    fn is_mouse_pressed(&self, key: MouseButton) -> bool;
    fn is_mouse_just_pressed(&self, key: MouseButton) -> bool;
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
    fn get_key_axis(&self, negative_key: KeyCode, positive_key: KeyCode) -> f32;
    fn get_axis(&self, negative_action: &str, positive_action: &str) -> f32;
}

pub trait EventApi {
    fn send<T: 'static>(&mut self, id: Id, message: T);
    fn send_boxed_any(&mut self, id: Id, message: Box<dyn Any + 'static>);
    fn emit<T: 'static>(&mut self, event: T);
    fn emit_targeted<T: 'static>(&mut self, id: Id, event: T);
    fn mailbox(&mut self) -> &mut IndexMap<Id, Vec<Box<dyn Any>>>;
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
    ) -> CollisionFlag;
    fn snap_to_floor(&mut self, my_id: Id, snap_length: f32) -> Option<f32>;
    fn translate_my_colliders(&mut self, my_id: Id, offset: Vector2);
}

pub trait RenderApi {
    fn draw(&mut self, z_index: u8, command: DrawCommand);
    fn draw_rect(&mut self, rect: Rect, color: Color, z_index: u8);
    fn draw_sprite(
        &mut self,
        position: Vector2,
        texture: Handler<ImageAsset>,
        anchor: Anchor,
        source: Option<Rect>,
        flip_v: bool,
        flip_h: bool,
        z_index: u8,
    );
    fn camera_mut(&mut self) -> &mut Vector2;
}
pub trait AssetApi {
    fn load_texture(&mut self, path: &str) -> Handler<ImageAsset>;
    fn load_texture_and_resize(
        &mut self,
        path: &str,
        width: u32,
        height: u32,
    ) -> Handler<ImageAsset>;
    fn load_audio(&mut self, path: &str) -> Handler<AudioAsset>;
}
pub trait AudioApi {
    fn play(&mut self, sound: Handler<AudioAsset>, looped: bool) -> Player;
}

pub trait SceneApi {
    fn push_scene<T: Scene + 'static>(&mut self, scene: T);
    fn change_scene<T: Scene + 'static>(&mut self, scene: T);
    fn pop_scene(&mut self);
    fn clear_scene(&mut self);
}
