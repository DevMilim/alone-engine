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

pub struct EventManager {
    pub mailboxes: FxHashMap<Id, Vec<GlobalEvent>>,
    pub broadcasts: FxHashMap<TypeId, Vec<Arc<dyn Any + Send + Sync + 'static>>>,
    pub broadcast_cursors: FxHashMap<(Id, TypeId), usize>,
    pub subscribers: FxHashMap<TypeId, FxHashSet<Id>>,
    pub aplication_commands: VecDeque<AppCommands>,

    pub broadcast_version: u64,
    pub broadcast_scratch_pool: Vec<Vec<Arc<dyn Any + Send + Sync + 'static>>>,
}

impl Default for EventManager {
    fn default() -> Self {
        Self {
            mailboxes: FxHashMap::default(),
            broadcasts: FxHashMap::default(),
            broadcast_cursors: FxHashMap::default(),
            subscribers: FxHashMap::default(),
            aplication_commands: VecDeque::new(),
            broadcast_version: 0,
            broadcast_scratch_pool: Vec::new(),
        }
    }
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
        self.broadcast_cursors
            .retain(|&(cursor_id, _), _| cursor_id != id);
    }

    pub fn poll_broadcasts(
        &mut self,
        id: Id,
        type_id: TypeId,
    ) -> Vec<Arc<dyn Any + Send + Sync + 'static>> {
        let mut buf = self.broadcast_scratch_pool.pop().unwrap_or_default();

        let Some(log) = self.broadcasts.get(&type_id) else {
            return buf;
        };

        let cursor = self.broadcast_cursors.entry((id, type_id)).or_insert(0);
        let start = (*cursor).min(log.len());
        *cursor = log.len();

        if start < log.len() {
            buf.extend(log[start..].iter().cloned());
        }

        buf
    }

    pub fn recycle_broadcast_buffer(&mut self, mut buf: Vec<Arc<dyn Any + Send + Sync + 'static>>) {
        buf.clear();
        self.broadcast_scratch_pool.push(buf);
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

    pub fn insert_broadcast_event(&mut self, event: Arc<dyn Any + Send + Sync + 'static>) {
        let type_id = (*event).type_id();

        if !self.subscribers.contains_key(&type_id) {
            return;
        }

        self.broadcasts.entry(type_id).or_default().push(event);
        self.broadcast_version = self.broadcast_version.wrapping_add(1);
    }

    pub fn prune_dead_mailboxes(&mut self, live_ids: &FxHashSet<Id>) {
        self.mailboxes.retain(|id, _| live_ids.contains(id));
    }

    pub fn clear_broadcast_log(&mut self) {
        self.broadcasts.clear();
        self.broadcast_cursors.clear();
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
