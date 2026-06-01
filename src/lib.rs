mod audio;
mod collision;
mod components;
mod core;
mod event;
mod input;
mod math;
mod render;
mod resources;
mod runtime;

pub use audio::*;
pub use collision::*;
pub use components::*;
pub use core::*;
pub use event::*;
pub use input::*;
pub use macros::*;
pub use math::*;
pub use render::*;
pub use resources::*;
pub use runtime::*;
use std::time::Instant;

pub struct TimeDebug {
    instant: Instant,
}

impl TimeDebug {
    pub fn start() -> Self {
        Self {
            instant: Instant::now(),
        }
    }
    pub fn larp(&mut self, message: &str) {
        let time = self.instant.elapsed();
        println!("Debug: {} time: {}", message, time.as_nanos());
        self.instant = Instant::now();
    }
    pub fn end(&self, message: &str) {
        let time = self.instant.elapsed();
        println!("Debug: {} time: {}", message, time.as_nanos());
    }
}
