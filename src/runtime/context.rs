use std::{any::Any, collections::VecDeque};

use indexmap::IndexMap;
use rodio::Player;
use winit::keyboard::KeyCode;

use crate::{
    AssetApi, AudioApi, AudioAsset, AudioSys, ColliderData, ColliderKey, CollisionApi,
    CollisionFlag, CollisionWorld, EngineApi, EventApi, GameObject, GlobalEvent, Handler, Id,
    ImageAsset, InputApi, InputState, Resources, RuntimeCommands, SpawnEvent, Vector2,
};

pub struct EngineContext<'a> {
    pub input: &'a InputState,
    pub event_queue: &'a mut VecDeque<RuntimeCommands>,
    pub events: &'a mut VecDeque<GlobalEvent>,
    pub mailbox: &'a mut IndexMap<Id, Vec<Box<dyn Any>>>,
    pub camera_pos: &'a mut Vector2,
    pub resources: &'a mut Resources,
    pub collision: &'a mut CollisionWorld,
    audio_sys: &'a AudioSys,
}
impl<'a> EngineApi for EngineContext<'a> {
    fn quit(&mut self) {
        self.event_queue.push_back(RuntimeCommands::Quit);
    }

    fn mailbox(&mut self) -> &mut IndexMap<Id, Vec<Box<dyn Any>>> {
        self.mailbox
    }

    fn camera_mut(&mut self) -> &mut Vector2 {
        &mut self.camera_pos
    }
}
impl<'a> EngineContext<'a> {
    pub fn new(
        input: &'a InputState,
        event_queue: &'a mut VecDeque<RuntimeCommands>,
        events: &'a mut VecDeque<GlobalEvent>,
        mailbox: &'a mut IndexMap<Id, Vec<Box<dyn Any>>>,
        camera_pos: &'a mut Vector2,
        resources: &'a mut Resources,
        collision: &'a mut CollisionWorld,
        audio_sys: &'a AudioSys,
    ) -> Self {
        Self {
            input,
            event_queue,
            events,
            mailbox,
            camera_pos,
            resources,
            collision,
            audio_sys,
        }
    }
}

impl<'a> AudioApi for EngineContext<'a> {
    fn load_audio(&mut self, path: &str) -> Handler<AudioAsset> {
        let sound_asset = AudioAsset::load_audio(path);
        self.resources.sounds.insert(path, sound_asset)
    }

    fn play(&mut self, sound: Handler<AudioAsset>, looped: bool) -> Player {
        self.audio_sys.play_controled(self.resources, sound, looped)
    }
}
impl<'a> AssetApi for EngineContext<'a> {
    fn load_texture(&mut self, path: &str) -> Handler<ImageAsset> {
        if let Some(id) = self.resources.textures.get_id(path) {
            return Handler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file(path);
        let id = self.resources.textures.insert(path, texture_asset);
        id
    }

    fn load_texture_and_resize(
        &mut self,
        path: &str,
        width: u32,
        height: u32,
    ) -> Handler<ImageAsset> {
        let path_key = path.to_string() + format!("#{width}x{height}").as_str();
        if let Some(id) = self.resources.textures.get_id(&path_key) {
            return Handler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file_and_resize(path, width, height);
        let id = self.resources.textures.insert(&path_key, texture_asset);
        id
    }
}
impl<'a> InputApi for EngineContext<'a> {
    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.input.is_key_pressed(key)
    }

    fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.input.is_key_just_pressed(key)
    }

    fn mouse_position(&self) -> Vector2 {
        self.input.mouse_position()
    }

    fn is_action_pressed(&self, action: &str) -> bool {
        self.input.is_action_pressed(action)
    }

    fn is_action_just_pressed(&self, action: &str) -> bool {
        self.input.is_action_just_pressed(action)
    }

    fn get_vector(
        &self,
        action_up: &str,
        action_down: &str,
        action_left: &str,
        action_right: &str,
    ) -> Vector2 {
        self.input
            .get_vector(action_up, action_down, action_left, action_right)
    }

    fn get_key_vector(
        &self,
        key_up: KeyCode,
        key_down: KeyCode,
        key_left: KeyCode,
        key_right: KeyCode,
    ) -> Vector2 {
        self.input
            .get_key_vector(key_up, key_down, key_left, key_right)
    }

    fn is_mouse_pressed(&self, key: winit::event::MouseButton) -> bool {
        self.input.is_mouse_pressed(key)
    }

    fn is_mouse_just_pressed(&self, key: winit::event::MouseButton) -> bool {
        self.input.is_mouse_just_pressed(key)
    }
}

impl<'a> EventApi for EngineContext<'a> {
    /// Utilizado para enviar uma mensagem endereçada para um GameObject especifico
    /// A mensagem tem que ser do mesmo tipo que o definido em type Message = T;
    fn send<T: 'static>(&mut self, id: Id, message: T) {
        let event = Box::new(message);
        self.mailbox.entry(id).or_default().push(event);
    }
    /// Utilizado para emitir um evento global que sera recebido por todos os GameObjects que definiram um #[game(subscribe(metodo: Tipo))]
    fn emit<T: 'static>(&mut self, event: T) {
        let event = GlobalEvent::Broadcast(Box::new(event));
        self.events.push_back(event);
    }
    /// Envia um evento similar a mensagem mas que pode ser de qualquer tipo, geralmente utilizado para comunicação de Componente para GameObject
    fn emit_targeted<T: 'static>(&mut self, id: Id, event: T) {
        let event = GlobalEvent::Targeted(id, Box::new(event));
        self.events.push_back(event);
    }
    fn spawn<T: GameObject + 'static>(&mut self, obj: T) {
        self.emit(SpawnEvent::new(obj));
    }

    fn mail_box_is_empty(&self) -> bool {
        self.mailbox.is_empty()
    }

    fn send_boxed_any(&mut self, id: Id, message: Box<dyn Any + 'static>) {
        self.mailbox.entry(id).or_default().push(message);
    }
}
impl<'a> CollisionApi for EngineContext<'a> {
    fn update_collider(&mut self, key: ColliderKey, data: ColliderData) {
        self.collision.update_collider(key, data);
    }

    fn remove_collider(&mut self, key: ColliderKey) {
        self.collision.remove_collider(key);
    }

    fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
    ) -> CollisionFlag {
        self.collision.move_and_slide(my_id, position, velocity)
    }
}
