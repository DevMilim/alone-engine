use std::{any::Any, sync::mpsc::Sender};

use indexmap::IndexMap;
use rodio::Player;
use winit::keyboard::KeyCode;

use crate::{
    AssetApi, AudioApi, AudioAsset, BackGroundEvent, ColliderData, ColliderKey, CollisionApi,
    CollisionFlag, CoreSystems, EngineApi, EventApi, EventManager, GameObject, GlobalEvent,
    Handler, Id, ImageAsset, InputApi,
    NetworkType::{Client, Server},
    ServerApi, ServerEvent, SpawnEvent, Vector2,
};

pub struct EngineContext<'a> {
    pub systems: &'a mut CoreSystems,
    pub events: &'a mut EventManager,
    pub camera_position: &'a mut Vector2,
    pub window_size: &'a (u32, u32),
    pub is_fixed_update: bool,
}

impl<'a> EngineApi for EngineContext<'a> {
    fn mailbox(&mut self) -> &mut IndexMap<Id, Vec<Box<dyn Any>>> {
        &mut self.events.mailbox
    }

    fn camera_mut(&mut self) -> &mut Vector2 {
        self.camera_position
    }

    fn window_size(&self) -> (u32, u32) {
        *self.window_size
    }

    fn async_ctx(&self) -> AsyncContext {
        AsyncContext {
            sender: self.systems.bg_event_sender.clone(),
        }
    }

    fn spawn_task<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.systems.async_handle.spawn(future);
    }
}
impl<'a> EngineContext<'a> {
    pub fn set_fixed_update(&mut self, value: bool) {
        self.is_fixed_update = value;
    }
}

impl<'a> AudioApi for EngineContext<'a> {
    fn load_audio(&mut self, path: &str) -> Handler<AudioAsset> {
        let sound_asset = AudioAsset::load_audio(path);
        self.systems.resources.sounds.insert(path, sound_asset)
    }

    fn play(&mut self, sound: Handler<AudioAsset>, looped: bool) -> Player {
        self.systems
            .audio
            .play_controled(&self.systems.resources, sound, looped)
    }
}
impl<'a> AssetApi for EngineContext<'a> {
    fn load_texture(&mut self, path: &str) -> Handler<ImageAsset> {
        if let Some(id) = self.systems.resources.textures.get_id(path) {
            return Handler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file(path);
        self.systems.resources.textures.insert(path, texture_asset)
    }

    fn load_texture_and_resize(
        &mut self,
        path: &str,
        width: u32,
        height: u32,
    ) -> Handler<ImageAsset> {
        let path_key = path.to_string() + format!("#{width}x{height}").as_str();
        if let Some(id) = self.systems.resources.textures.get_id(&path_key) {
            return Handler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file_and_resize(path, width, height);
        self.systems
            .resources
            .textures
            .insert(&path_key, texture_asset)
    }
}
impl<'a> InputApi for EngineContext<'a> {
    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.systems.input.is_key_pressed(key)
    }

    fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.systems
            .input
            .is_key_just_pressed(key, self.is_fixed_update)
    }

    fn mouse_position(&self) -> Vector2 {
        self.systems.input.mouse_position()
    }

    fn is_action_pressed(&self, action: &str) -> bool {
        self.systems.input.is_action_pressed(action)
    }

    fn is_action_just_pressed(&self, action: &str) -> bool {
        self.systems
            .input
            .is_action_just_pressed(action, self.is_fixed_update)
    }

    fn get_vector(
        &self,
        action_up: &str,
        action_down: &str,
        action_left: &str,
        action_right: &str,
    ) -> Vector2 {
        self.systems
            .input
            .get_vector(action_up, action_down, action_left, action_right)
    }

    fn get_key_vector(
        &self,
        key_up: KeyCode,
        key_down: KeyCode,
        key_left: KeyCode,
        key_right: KeyCode,
    ) -> Vector2 {
        self.systems
            .input
            .get_key_vector(key_up, key_down, key_left, key_right)
    }

    fn is_mouse_pressed(&self, key: winit::event::MouseButton) -> bool {
        self.systems.input.is_mouse_pressed(key)
    }

    fn is_mouse_just_pressed(&self, key: winit::event::MouseButton) -> bool {
        self.systems
            .input
            .is_mouse_just_pressed(key, self.is_fixed_update)
    }

    fn get_key_axis(&self, negative_key: KeyCode, positive_key: KeyCode) -> f32 {
        self.systems.input.get_key_axis(negative_key, positive_key)
    }

    fn get_axis(&self, negative_action: &str, positive_action: &str) -> f32 {
        self.systems
            .input
            .get_axis(negative_action, positive_action)
    }
}

impl<'a> EventApi for EngineContext<'a> {
    /// Utilizado para enviar uma mensagem endereçada para um GameObject especifico
    /// A mensagem tem que ser do mesmo tipo que o definido em type Message = T;
    fn send<T: 'static>(&mut self, id: Id, message: T) {
        let event = Box::new(message);
        self.events.insert_mailbox(id, event);
    }
    /// Utilizado para emitir um evento global que sera recebido por todos os GameObjects que definiram um #[game(subscribe(metodo: Tipo))]
    fn emit<T: 'static>(&mut self, event: T) {
        let event = GlobalEvent::Broadcast(Box::new(event));
        self.events.insert_global_event(event);
    }
    /// Envia um evento similar a mensagem mas que pode ser de qualquer tipo, geralmente utilizado para comunicação de Componente para GameObject
    fn emit_targeted<T: 'static>(&mut self, id: Id, event: T) {
        let event = GlobalEvent::Targeted(id, Box::new(event));
        self.events.insert_global_event(event);
    }
    fn spawn<T: GameObject + 'static>(&mut self, obj: T) {
        self.emit(SpawnEvent::new(obj));
    }

    fn mail_box_is_empty(&self) -> bool {
        self.events.mailbox.is_empty()
    }

    fn send_boxed_any(&mut self, id: Id, message: Box<dyn Any + 'static>) {
        self.events.insert_mailbox_boxed_any(id, message);
    }
}
impl<'a> CollisionApi for EngineContext<'a> {
    fn update_collider(&mut self, key: ColliderKey, data: ColliderData) {
        self.systems.collision.update_collider(key, data);
    }

    fn remove_collider(&mut self, key: ColliderKey) {
        self.systems.collision.remove_collider(key);
    }

    fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
    ) -> CollisionFlag {
        self.systems
            .collision
            .move_and_slide(my_id, position, velocity)
    }

    fn snap_to_floor(&mut self, my_id: Id, snap_length: f32) -> Option<f32> {
        self.systems.collision.snap_to_floor(my_id, snap_length)
    }

    fn translate_my_colliders(&mut self, my_id: Id, offset: Vector2) {
        self.systems.collision.translate_my_colliders(my_id, offset);
    }
}

pub struct AsyncContext {
    sender: Sender<BackGroundEvent>,
}

impl AsyncContext {
    pub fn emit<T: Any + Send + 'static>(&self, event: T) {
        let _ = self
            .sender
            .send(BackGroundEvent::Broadcast(Box::new(event)));
    }
    pub fn emit_targeted<T: Any + Send + 'static>(&self, id: Id, event: T) {
        let _ = self
            .sender
            .send(BackGroundEvent::Targeted(id, Box::new(event)));
    }
    pub fn send<T: Any + Send + 'static>(&self, id: Id, message: T) {
        let _ = self
            .sender
            .send(BackGroundEvent::Send(id, Box::new(message)));
    }
}
impl<'a> ServerApi for EngineContext<'a> {
    fn send_to_server(&mut self, message: ServerEvent) {
        if let Client(client) = &mut self.systems.network {
            client.send(message);
        } else {
            eprintln!("Tentando enviar dados ao servidor, mas este no não e um cliente.")
        }
    }

    fn send_to_client(&mut self, target: std::net::SocketAddr, message: ServerEvent) {
        if let Server(client) = &mut self.systems.network {
            client.send(target, message);
        } else {
            eprintln!("Tentando enviar dados ao cliente, mas este no não e um servidor.")
        }
    }
}
