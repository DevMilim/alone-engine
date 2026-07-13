mod api;
mod base;
mod core;
mod handler;
mod ldtk_api;

pub use api::*;
pub use base::*;
pub use core::*;
pub use handler::*;
pub use ldtk_api::*;
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender, channel},
};
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};

use crate::{
    audio::AudioSys,
    collision::{ColliderKey, CollisionWorld},
    event::{BackGroundEvent, GlobalEvent, TriggerEvent, TriggerKind},
    input::InputState,
    network::{NetworkClient, NetworkServer},
    resources::Resources,
};

pub enum NetworkType {
    None,
    Server(NetworkServer),
    Client(NetworkClient),
}

pub struct CoreSystems {
    pub audio: AudioSys,
    pub resources: Resources,
    pub collision: CollisionWorld,
    pub input: InputState,
    pub async_handle: Handle,
    pub network: NetworkType,

    pub bg_event_sender: Sender<BackGroundEvent>,
    pub bg_event_receiver: Receiver<BackGroundEvent>,
    pub task_handles: HashMap<Id, Vec<JoinHandle<()>>>,
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
            input: InputState::default(),
            network: NetworkType::None,
            async_handle,
            bg_event_sender: bg_tx,
            bg_event_receiver: bg_rx,
            task_handles: HashMap::new(),
        }
    }
}

impl CoreSystems {
    pub fn collision_step(&mut self) -> Vec<GlobalEvent> {
        self.collision.step();

        let mut trigger_events =
            self.emit_trigger_events(self.collision.get_entered_pairs(), TriggerKind::Enter);
        trigger_events
            .extend(self.emit_trigger_events(self.collision.get_exited_pairs(), TriggerKind::Exit));

        self.collision.commit();
        trigger_events
    }
    fn emit_trigger_events(
        &self,
        pairs: Vec<(ColliderKey, ColliderKey)>,
        kind: TriggerKind,
    ) -> Vec<GlobalEvent> {
        let mut trigger_events = Vec::new();
        for (a, b) in pairs {
            if a.id == b.id {
                continue;
            }
            let (Some(da), Some(db)) = (
                self.collision.colliders.get(&a),
                self.collision.colliders.get(&b),
            ) else {
                continue;
            };

            if da.is_sensor {
                let ev = TriggerEvent {
                    owner: b.id,
                    sensor: a,
                    kind,
                };
                trigger_events.push(GlobalEvent::Targeted(a.id, Box::new(ev)));
                trigger_events.push(GlobalEvent::Targeted(b.id, Box::new(ev)));
            }
            if db.is_sensor {
                let ev = TriggerEvent {
                    owner: a.id,
                    sensor: b,
                    kind,
                };
                trigger_events.push(GlobalEvent::Targeted(b.id, Box::new(ev)));
                trigger_events.push(GlobalEvent::Targeted(a.id, Box::new(ev)));
            }
        }
        trigger_events
    }
}
