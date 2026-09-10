use std::any::TypeId;

use std::sync::{Arc, Mutex};
use std::{any::Any, collections::VecDeque};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::collision::ColliderKey;
use crate::core::{GameObject, Id};
use crate::runtime::AppCommands;

pub type CallbackEvent = Box<dyn Fn() -> Box<dyn Any + Send + 'static>>;

#[derive(Debug)]
pub enum GlobalEvent {
    Targeted(Id, Box<dyn Any + Send + 'static>),
    Send(Id, Box<dyn Any + Send + 'static>),
    Broadcast(Arc<dyn Any + Send + Sync + 'static>),
}

impl Default for EventManager {
    fn default() -> Self {
        Self {
            mailboxes: FxHashMap::default(),
            subscribers: FxHashMap::default(),
            aplication_commands: VecDeque::new(),
        }
    }
}

pub struct EventManager {
    pub mailboxes: FxHashMap<Id, Vec<GlobalEvent>>,
    pub subscribers: FxHashMap<TypeId, FxHashSet<Id>>,
    pub aplication_commands: VecDeque<AppCommands>,
}

impl EventManager {
    pub fn take_mailbox(&mut self, id: Id) -> Option<Vec<GlobalEvent>> {
        self.mailboxes.remove(&id)
    }
    pub fn register_subscriptions(&mut self, id: Id, types: &[TypeId]) {
        for &type_id in types {
            self.subscribers.entry(type_id).or_default().insert(id);
        }
    }

    pub fn unregister_subscriptions(&mut self, id: Id, types: &[TypeId]) {
        for &type_id in types {
            if let Some(ids) = self.subscribers.get_mut(&type_id) {
                ids.remove(&id);

                if ids.is_empty() {
                    self.subscribers.remove(&type_id);
                }
            }
        }
    }
    pub fn insert_global_event(&mut self, event: GlobalEvent) {
        match event {
            GlobalEvent::Broadcast(payload) => self.insert_broadcast_event(payload),
            GlobalEvent::Targeted(id, payload) => {
                self.mailboxes
                    .entry(id)
                    .or_default()
                    .push(GlobalEvent::Targeted(id, payload));
            }
            GlobalEvent::Send(id, payload) => {
                self.mailboxes
                    .entry(id)
                    .or_default()
                    .push(GlobalEvent::Send(id, payload));
            }
        }
    }
    pub fn insert_broadcast_event(&mut self, event: Arc<dyn Any + Send + Sync>) {
        let type_id = (*event).type_id();

        let Some(subscriber_ids) = self.subscribers.get(&type_id) else {
            return;
        };

        for &id in subscriber_ids {
            self.mailboxes
                .entry(id)
                .or_default()
                .push(GlobalEvent::Broadcast(event.clone()));
        }
    }
    pub fn prune_dead_mailboxes(&mut self, live_ids: &FxHashSet<Id>) {
        self.mailboxes.retain(|id, _| live_ids.contains(id));
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
#[derive(Debug, Clone, Copy)]
pub struct TriggerEvent {
    pub owner: Id,
    pub sensor: ColliderKey,
    pub kind: TriggerKind,
}

/// Utilizado para definir se ocorreu uma entrada ou saida de algum colisor
/// Como Entrada e Saida
#[derive(Debug, Clone, Copy)]
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
///     #[object]
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
    payload: Mutex<Option<T>>,
}

impl<T: GameObject> SpawnEvent<T> {
    pub fn new(obj: T) -> Self {
        Self {
            payload: Mutex::new(Some(obj)),
        }
    }
    pub fn take(&self) -> Option<T> {
        self.payload.lock().ok()?.take()
    }
}
