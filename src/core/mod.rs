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

use crate::{AudioSys, CollisionWorld, InputState, Resources, TriggerEvent, TriggerKind};

#[derive(Default)]
pub struct CoreSystems {
    pub audio: AudioSys,
    pub resources: Resources,
    pub collision: CollisionWorld,
    pub input: InputState,
}

impl CoreSystems {
    pub fn collision_step(&mut self) -> Vec<GlobalEvent> {
        self.collision.step();
        let mut trigger_events = Vec::new();
        for (a, b) in self.collision.get_entered_pairs() {
            if a.id == b.id {
                continue;
            }
            let da = self.collision.colliders.get(&a).unwrap();
            let db = self.collision.colliders.get(&b).unwrap();

            if da.is_sensor {
                let ev = TriggerEvent {
                    owner: b.id,
                    sensor: a.clone(),
                    kind: TriggerKind::Enter,
                };
                trigger_events.push(GlobalEvent::Targeted(a.id, Box::new(ev.clone())));
                trigger_events.push(GlobalEvent::Targeted(b.id, Box::new(ev)));
            }
            if db.is_sensor {
                let ev = TriggerEvent {
                    owner: a.id,
                    sensor: b.clone(),
                    kind: TriggerKind::Enter,
                };
                trigger_events.push(GlobalEvent::Targeted(b.id, Box::new(ev.clone())));
                trigger_events.push(GlobalEvent::Targeted(a.id, Box::new(ev)));
            }
        }
        for (a, b) in self.collision.get_exited_pairs() {
            if a.id == b.id {
                continue;
            }
            let da = self.collision.colliders.get(&a).unwrap();
            let db = self.collision.colliders.get(&b).unwrap();

            if da.is_sensor {
                let ev = TriggerEvent {
                    owner: b.id,
                    sensor: a.clone(),
                    kind: TriggerKind::Exit,
                };
                trigger_events.push(GlobalEvent::Targeted(a.id, Box::new(ev.clone())));
                trigger_events.push(GlobalEvent::Targeted(b.id, Box::new(ev)));
            }
            if db.is_sensor {
                let ev = TriggerEvent {
                    owner: a.id,
                    sensor: b.clone(),
                    kind: TriggerKind::Exit,
                };
                trigger_events.push(GlobalEvent::Targeted(b.id, Box::new(ev.clone())));
                trigger_events.push(GlobalEvent::Targeted(a.id, Box::new(ev)));
            }
        }
        self.collision.commit();
        trigger_events
    }
}
