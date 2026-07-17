use std::{
    any::Any,
    time::{Duration, Instant},
};

use crate::core::{Base, Component, EngineApi};

pub enum TimerEvent {
    Timeout,
}

pub struct Timer {
    instant: Option<Instant>,
    duration: Duration,
    event: Option<Box<dyn Fn() -> Box<dyn Any + 'static>>>,
    repeat: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            instant: None,
            duration: Duration::from_secs(1),
            event: None,
            repeat: true,
        }
    }
    pub fn now(&mut self) {
        self.instant = Some(Instant::now());
    }
    pub fn set_event<T: Clone + 'static>(&mut self, event: T) {
        self.event = Some(Box::new(move || Box::new(event.clone())));
    }
    pub fn start_timer(&mut self, duration: Duration, repeat: bool) {
        self.duration = duration;
        self.repeat = repeat;
        self.instant = Some(Instant::now())
    }
    pub fn stop(&mut self) {
        self.instant = None;
    }
}

impl Component for Timer {
    fn update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, _delta: f32) {
        if let Some(instant) = self.instant
            && instant.elapsed() >= self.duration
        {
            if self.repeat {
                self.instant = Some(instant + self.duration);
            } else {
                self.stop();
            }
            if let Some(event) = &self.event {
                ctx.send_boxed_any(base.id, event());
            } else {
                ctx.emit_targeted(base.id, TimerEvent::Timeout);
            }
        }
    }
}
