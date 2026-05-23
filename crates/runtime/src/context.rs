use std::{any::Any, collections::VecDeque};

use assets::{ImageAsset, Resources};
use core::{
    AssetApi, AudioApi, EngineApi, EventApi, GameObject, GlobalEvent, Id, InputApi, RenderApi,
    Vector2,
};
use core::{DrawCommand, TextureHandler};
use event::SpawnEvent;
use indexmap::IndexMap;
use input::{InputState, KeyCode};
use render::RenderCommands;

pub struct EngineContext<'a> {
    pub input: &'a InputState,
    pub event_queue: &'a mut VecDeque<RenderCommands>,
    pub events: &'a mut VecDeque<GlobalEvent>,
    pub mailbox: &'a mut IndexMap<Id, Vec<Box<dyn Any>>>,
    pub camera_pos: &'a mut Vector2,
    pub resources: &'a mut Resources,
}
impl<'a> EngineApi for EngineContext<'a> {
    fn quit(&mut self) {
        self.event_queue.push_back(RenderCommands::Quit);
    }

    fn mailbox(&mut self) -> &mut IndexMap<Id, Vec<Box<dyn Any>>> {
        self.mailbox
    }
}
impl<'a> EngineContext<'a> {
    pub fn new(
        input: &'a InputState,
        event_queue: &'a mut VecDeque<RenderCommands>,
        events: &'a mut VecDeque<GlobalEvent>,
        mailbox: &'a mut IndexMap<Id, Vec<Box<dyn Any>>>,
        camera_pos: &'a mut Vector2,
        resources: &'a mut Resources,
    ) -> Self {
        Self {
            input,
            event_queue,
            events,
            mailbox,
            camera_pos,
            resources,
        }
    }
}

impl<'a> AudioApi for EngineContext<'a> {}
impl<'a> AssetApi for EngineContext<'a> {
    fn load_texture(&mut self, path: &str) -> TextureHandler {
        if let Some(id) = self.resources.textures.get_id(path) {
            return TextureHandler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file(path);
        let id = self.resources.textures.insert(path, texture_asset);
        TextureHandler::new(id)
    }

    fn load_texture_and_resize(&mut self, path: &str, width: u32, height: u32) -> TextureHandler {
        if let Some(id) = self.resources.textures.get_id(path) {
            return TextureHandler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file_and_resize(path, width, height);
        let id = self.resources.textures.insert(path, texture_asset);
        TextureHandler::new(id)
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
}
