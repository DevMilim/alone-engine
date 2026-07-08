use std::cell::RefCell;

use std::net::SocketAddr;
use std::{any::Any, collections::VecDeque};

use bincode::{Decode, Encode};
use indexmap::IndexMap;

use crate::collision::ColliderKey;
use crate::core::{GameObject, Id};

#[derive(Debug, Encode, Decode, Clone)]
pub enum ServerEvent {
    Broadcast(Vec<u8>),
    Targeted(Id, Vec<u8>),
    Send(Id, Vec<u8>),
}

pub struct NetworkMessage {
    pub event: ServerEvent,
    pub socket: Option<SocketAddr>,
}
impl NetworkMessage {
    pub fn new(event: ServerEvent, socket: Option<SocketAddr>) -> Self {
        Self { event, socket }
    }
}

impl ServerEvent {
    pub fn downcast_ref<T: Decode<()> + Encode + 'static>(&self) -> Option<T> {
        let bytes = self.clone().get_event();
        crate::deserialize_bytes::<T>(&bytes)
    }

    pub fn get_event(self) -> Vec<u8> {
        match self {
            ServerEvent::Broadcast(items) => items,
            ServerEvent::Targeted(_, items) => items,
            ServerEvent::Send(_, items) => items,
        }
    }
}

#[derive(Debug)]
pub enum GlobalEvent {
    Broadcast(Box<dyn Any>),
    Targeted(Id, Box<dyn Any>),
}

#[derive(Debug)]
pub enum BackGroundEvent {
    Broadcast(Box<dyn Any + Send + 'static>),
    Targeted(Id, Box<dyn Any + Send + 'static>),
    Send(Id, Box<dyn Any + Send + 'static>),
}

/// Gerenciador e container de eventos como mensagens diretas e eventos globais como subscribe
#[derive(Debug)]
pub struct EventManager {
    pub global_events: VecDeque<GlobalEvent>,
    pub mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,

    pub global_server_events: VecDeque<ServerEvent>,
    pub server_mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,
}

impl Default for EventManager {
    fn default() -> Self {
        Self {
            global_events: VecDeque::new(),
            mailbox: IndexMap::new(),
            global_server_events: VecDeque::new(),
            server_mailbox: IndexMap::new(),
        }
    }
}

impl EventManager {
    /// Insere um evento global a fila de eventos que sera processado pelos GameObjects com subscribe
    /// pode ser utilizado com
    /// Para utilizar ele deve ser utilizada a assinatura
    /// ```
    /// #[derive(GameObject)]
    /// #[game(connect(event: Event))]
    /// pub struct Player {
    ///     #[base]
    ///     base: Base,
    /// }
    /// impl Player {
    ///     fn event(&mut self, _ctx: &mut impl EngineApi, event: &Event) {
    ///         println!("Evento global recebido")
    ///     }
    /// }
    ///
    /// ```
    pub fn insert_global_event(&mut self, event: GlobalEvent) {
        self.global_events.push_back(event);
    }
    /// Insere uma mensagem a fila de caixa de mensagens
    /// Ao cair aqui a engine ira encaminhar para o GameObject com o id especificado
    /// Se o tipo enviado for compativel com
    /// ```
    /// type Message = T
    /// ```
    /// Se a mensagem for compativel o GameObject ira receber ela em
    /// ```
    /// fn on_message(&mut self, ctx: &mut impl EngineApi, msg: Self::Message){}
    /// ```
    pub fn insert_mailbox<T: 'static>(&mut self, id: Id, mail: T) {
        self.mailbox.entry(id).or_default().push(Box::new(mail));
    }
    /// Insere uma mensagem na caixa de mensagens mas com mensagem com tipo `Box<dyn Any + 'static>`
    pub fn insert_mailbox_boxed_any(&mut self, id: Id, message: Box<dyn Any + 'static>) {
        self.mailbox.entry(id).or_default().push(message);
    }
}

/// Evento usado para Collider com `is_sensor: true`
/// Para utilizar ele deve ser utilizada a assinatura
/// ```
/// #[derive(GameObject)]
/// #[game(connect(on_trigger_event: TriggerEvent))]
/// pub struct Player {
///     #[base]
///     base: Base,
///     #[component]
///     collider: Collider,
/// }
/// impl Player {
///     fn on_trigger_event(&mut self, _ctx: &mut impl EngineApi, event: &TriggerEvent) {
///         println!("Evento de colisão recebido")
///     }
/// }
///
/// ```
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub owner: Id,
    pub sensor: ColliderKey,
    pub kind: TriggerKind,
}

/// Utilizado para definir se ocorreu uma entrada ou saida de algum colisor
/// Como Entrada e Saida
#[derive(Debug, Clone)]
pub enum TriggerKind {
    Enter,
    Exit,
}
/// Evento utilizado para Spawn
/// Exemplo:
/// ```
/// #[derive(GameObject)]
/// #[game(subscribe(spawn_bullet: SpawnEvent<Bullet>))]
/// pub struct MainScene {
///     #[base]
///     base: Base,
///     #[component]
///     bullets: Vec<Bullet>,
/// }
/// impl MainScene {
///     fn spawn_bullet(&mut self, _ctx: &mut impl EngineApi, event: &SpawnEvent<Bullet>) {
///         self.bullets.push(event.take().expect("Erro ao spawnar bullet"));
///     }
/// }
///
/// ```
pub struct SpawnEvent<T> {
    payload: RefCell<Option<T>>,
}

impl<T: GameObject> SpawnEvent<T> {
    pub fn new(obj: T) -> Self {
        Self {
            payload: RefCell::new(Some(obj)),
        }
    }
    pub fn take(&self) -> Option<T> {
        self.payload.borrow_mut().take()
    }
}
