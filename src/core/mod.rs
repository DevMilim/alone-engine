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

use crate::{
    AudioSys, CollisionWorld, InputState, InputType, Resources, RuntimeEvent, TriggerEvent,
    TriggerKind,
};

#[derive(Default)]
pub struct CoreSystems {
    pub audio: AudioSys,
    pub resources: Resources,
    pub collision: CollisionWorld,
    pub input: InputState,
}


impl CoreSystems {
    pub fn events(&mut self, cmd: RuntimeEvent) {
        match cmd {
            RuntimeEvent::KeyDown(keycode) => {
                self.input.update_input_state(InputType::Key(keycode), true)
            }
            RuntimeEvent::KeyUp(keycode) => self
                .input
                .update_input_state(InputType::Key(keycode), false),
            RuntimeEvent::MousePosition(x, y) => self.input.set_mouse_position(x, y),
            RuntimeEvent::MouseInput(mouse_button, is_pressed) => self
                .input
                .update_input_state(InputType::Mouse(mouse_button), is_pressed),
            _ => {}
        }
    }
    pub fn collision_step(&mut self) -> Vec<GlobalEvent> {
        self.collision.step();
        let mut trigger_events = Vec::new();
        for (a, b) in self.collision.get_entered_pairs() {
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
