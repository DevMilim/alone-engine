mod api;
mod base;
mod core;
mod handler;
mod ldtk_api;
mod pool;
mod slot;

pub use api::*;
pub use base::*;
pub use core::*;
pub use handler::*;
pub use ldtk_api::*;
pub use pool::*;
pub use slot::*;

use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    any::{Any, TypeId},
    sync::{
        LazyLock,
        mpsc::{Receiver, Sender, channel},
    },
};
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};

use crate::{
    audio::AudioSys,
    collision::{ColliderKey, CollisionWorld},
    event::{BackGroundEvent, EventManager, GlobalEvent, TriggerEvent, TriggerKind},
    input::InputState,
    resources::Resources,
};

pub static EMPTY_BASE: LazyLock<Base> = LazyLock::new(Base::default);

pub struct TriggerCallbacks {
    pub on_enter: Option<Box<dyn Fn() -> Box<dyn Any + 'static>>>,
    pub on_exit: Option<Box<dyn Fn() -> Box<dyn Any + 'static>>>,
}

pub struct CoreSystems {
    pub audio: AudioSys,
    pub resources: Resources,
    pub collision: CollisionWorld,
    pub trigger_callbacks: FxHashMap<ColliderKey, TriggerCallbacks>,
    pub input: InputState,
    pub async_handle: Handle,

    pub bg_event_sender: Sender<BackGroundEvent>,
    pub bg_event_receiver: Receiver<BackGroundEvent>,
    pub task_handles: FxHashMap<Id, Vec<JoinHandle<()>>>,
    pub service_register: FxHashMap<TypeId, Id>,
    pub live_ids: FxHashSet<Id>,
}

impl Default for CoreSystems {
    fn default() -> Self {
        let (handle_tx, handle_rx) = channel::<Handle>();

        std::thread::spawn(move || {
            let rt = Runtime::new().expect("Falha ao criar Runtime");

            handle_tx.send(rt.handle().clone()).unwrap();

            rt.block_on(async { std::future::pending::<()>().await });
        });
        let async_handle = handle_rx.recv().expect("Falha ao receber Handle");
        let (bg_tx, bg_rx) = channel::<BackGroundEvent>();
        Self {
            audio: AudioSys::default(),
            resources: Resources::default(),
            collision: CollisionWorld::default(),
            trigger_callbacks: FxHashMap::default(),
            input: InputState::default(),
            async_handle,
            bg_event_sender: bg_tx,
            bg_event_receiver: bg_rx,
            task_handles: FxHashMap::default(),
            service_register: FxHashMap::default(),
            live_ids: FxHashSet::default(),
        }
    }
}

impl CoreSystems {
    pub fn collision_step(&mut self, events: &mut EventManager) -> Vec<GlobalEvent> {
        self.collision.step();

        let mut trigger_events = self.emit_trigger_events(
            events,
            self.collision.get_entered_pairs(),
            TriggerKind::Enter,
        );
        trigger_events.extend(self.emit_trigger_events(
            events,
            self.collision.get_exited_pairs(),
            TriggerKind::Exit,
        ));

        self.collision.commit();
        trigger_events
    }
    fn emit_trigger_events(
        &self,
        events: &mut EventManager,
        pairs: Vec<(ColliderKey, ColliderKey)>,
        kind: TriggerKind,
    ) -> Vec<GlobalEvent> {
        let mut trigger_events = Vec::new();

        for (a, b) in pairs {
            if a.id == b.id {
                continue;
            }

            let (Some(da), Some(db)) = (self.collision.get(&a), self.collision.get(&b)) else {
                continue;
            };

            if da.is_sensor {
                self.emit_trigger(events, &mut trigger_events, a, b.id, kind);
            }

            if db.is_sensor {
                self.emit_trigger(events, &mut trigger_events, b, a.id, kind);
            }
        }

        trigger_events
    }
    fn emit_trigger(
        &self,
        events: &mut EventManager,
        trigger_events: &mut Vec<GlobalEvent>,
        sensor: ColliderKey,
        owner: Id,
        kind: TriggerKind,
    ) {
        if let Some(cb) = self.trigger_callbacks.get(&sensor) {
            let msg = match kind {
                TriggerKind::Enter => cb.on_enter.as_ref().map(|f| f()),
                TriggerKind::Exit => cb.on_exit.as_ref().map(|f| f()),
            };

            if let Some(msg) = msg {
                events.insert_mailbox_boxed_any(sensor.id, msg);
                return;
            }
        }

        let ev = TriggerEvent {
            owner,
            sensor,
            kind,
        };

        trigger_events.push(GlobalEvent::Targeted(sensor.id, Box::new(ev)));
    }
}
