use std::{any::Any, collections::VecDeque};

use assets::{ImageAsset, Resources};
use core::{EngineApi, GameObject, GameObjectDispatch, GlobalEvent, Handler, Id, Vector2};
use event::SpawnEvent;
use indexmap::IndexMap;
use input::InputState;
use render::DrawCommand;

use crate::engine::EngineCommands;

pub struct EngineContext<'a> {
    pub input: &'a InputState,
    pub event_queue: &'a mut VecDeque<EngineCommands>,
    pub events: &'a mut VecDeque<GlobalEvent>,
    pub mailbox: &'a mut IndexMap<Id, Vec<Box<dyn Any>>>,
    pub camera_pos: &'a mut Vector2,
    pub resources: &'a mut Resources,
    pub draw_queue: &'a mut Vec<DrawCommand>,
}
impl<'a> EngineApi for EngineContext<'a> {}
impl<'a> EngineContext<'a> {
    pub fn new(
        input: &'a InputState,
        event_queue: &'a mut VecDeque<EngineCommands>,
        events: &'a mut VecDeque<GlobalEvent>,
        mailbox: &'a mut IndexMap<Id, Vec<Box<dyn Any>>>,
        camera_pos: &'a mut Vector2,
        resources: &'a mut Resources,
        draw_queue: &'a mut Vec<DrawCommand>,
    ) -> Self {
        Self {
            input,
            event_queue,
            events,
            mailbox,
            camera_pos,
            resources,
            draw_queue,
        }
    }

    /// Utilizado para enviar uma mensagem endereçada para um GameObject especifico
    /// A mensagem tem que ser do mesmo tipo que o definido em type Message = T;
    pub fn send<T: 'static>(&mut self, id: Id, message: T) {
        let event = Box::new(message);
        self.mailbox.entry(id).or_default().push(event);
    }
    /// Utilizado para emitir um evento global que sera recebido por todos os GameObjects que definiram um #[game(subscribe(metodo: Tipo))]
    pub fn emit<T: 'static>(&mut self, event: T) {
        let event = GlobalEvent::Broadcast(Box::new(event));
        self.events.push_back(event);
    }
    /// Envia um evento similar a mensagem mas que pode ser de qualquer tipo, geralmente utilizado para comunicação de Componente para GameObject
    pub fn emit_targeted<T: 'static>(&mut self, id: Id, event: T) {
        let event = GlobalEvent::Targeted(id, Box::new(event));
        self.events.push_back(event);
    }
    pub fn spawn<T: GameObject + GameObjectDispatch + 'static>(&mut self, obj: T) {
        self.emit(SpawnEvent::new(obj));
    }
    pub fn draw(&mut self, command: DrawCommand) {
        self.draw_queue.push(command);
    }
    pub fn load_texture(&mut self, path: &str) -> Handler<ImageAsset> {
        if let Some(id) = self.resources.textures.get_id(path) {
            return Handler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file(path);
        self.resources.textures.insert(path, texture_asset)
    }
    pub fn quit(&mut self) {
        self.event_queue.push_back(EngineCommands::Quit);
    }
}
