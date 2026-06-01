use std::any::Any;

use indexmap::IndexMap;
use rodio::Player;
use winit::keyboard::KeyCode;

use crate::{
    AssetApi, AudioApi, AudioAsset, ColliderData, ColliderKey, CollisionApi, CollisionFlag,
    CoreSystems, EngineApi, EventApi, EventManager, GameObject, GlobalEvent, Handler, Id,
    ImageAsset, InputApi, SpawnEvent, Vector2,
};

pub struct EngineContext<'a> {
    pub systems: &'a mut CoreSystems,
    pub events: &'a mut EventManager,
    pub camera_position: &'a mut Vector2,
    pub window_size: &'a (u32, u32),
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
}
impl<'a> EngineContext<'a> {}

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
        self.systems.input.is_key_just_pressed(key)
    }

    fn mouse_position(&self) -> Vector2 {
        self.systems.input.mouse_position()
    }

    fn is_action_pressed(&self, action: &str) -> bool {
        self.systems.input.is_action_pressed(action)
    }

    fn is_action_just_pressed(&self, action: &str) -> bool {
        self.systems.input.is_action_just_pressed(action)
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
        self.systems.input.is_mouse_just_pressed(key)
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
}
