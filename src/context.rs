use std::{any::Any, collections::VecDeque};

use indexmap::IndexMap;

use crate::{
    EngineCommands, GameObject, GameObjectDispatch, GlobalEvent, Id, InputState, Resources,
    SpawnEvent, Vector2,
};

pub struct EngineContext<'a> {
    pub input: &'a InputState,
    pub event_queue: &'a mut VecDeque<EngineCommands>,
    pub events: &'a mut VecDeque<GlobalEvent>,
    pub mailbox: &'a mut IndexMap<Id, Vec<Box<dyn Any>>>,
    pub camera_pos: &'a mut Vector2,
    pub resources: &'a mut Resources,
}
impl<'a> EngineContext<'a> {
    pub fn new(
        input: &'a InputState,
        event_queue: &'a mut VecDeque<EngineCommands>,
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
    pub fn quit(&mut self) {
        self.event_queue.push_back(EngineCommands::Quit);
    }
}
