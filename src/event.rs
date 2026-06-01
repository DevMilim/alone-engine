use std::cell::RefCell;
use winit::{event::MouseButton, keyboard::KeyCode};

use std::{any::Any, collections::VecDeque};

use indexmap::IndexMap;

use crate::{ColliderKey, GameObject, GlobalEvent, Id};

pub struct EventManager {
    pub runtime_events: VecDeque<RuntimeEvent>,
    pub global_events: VecDeque<GlobalEvent>,
    pub mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,
}

impl Default for EventManager {
    fn default() -> Self {
        Self {
            runtime_events: VecDeque::new(),
            global_events: VecDeque::new(),
            mailbox: IndexMap::new(),
        }
    }
}

impl EventManager {
    pub fn insert_runtime_event(&mut self, event: RuntimeEvent) {
        self.runtime_events.push_back(event);
    }
    pub fn insert_global_event(&mut self, event: GlobalEvent) {
        self.global_events.push_back(event);
    }
    pub fn insert_mailbox<T: 'static>(&mut self, id: Id, mail: T) {
        self.mailbox.entry(id).or_default().push(Box::new(mail));
    }
    pub fn insert_mailbox_boxed_any(&mut self, id: Id, message: Box<dyn Any + 'static>) {
        self.mailbox.entry(id).or_default().push(message);
    }
}

#[derive(Debug, Clone)]
pub enum TriggerKind {
    Enter,
    Exit,
}
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub owner: Id,
    pub sensor: ColliderKey,
    pub kind: TriggerKind,
}
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

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    MouseInput(MouseButton, bool),
    MousePosition(f32, f32),
    Quit,
}
