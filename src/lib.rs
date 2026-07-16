extern crate self as alone_engine;

pub mod audio;
pub mod collision;
pub mod components;
pub mod core;
pub mod event;
pub mod input;
pub mod math;
pub mod objects;
pub mod render;
pub mod resources;
pub mod runtime;
pub mod ui;

pub use macros::*;

pub mod prelude {
    pub use crate::audio::*;
    pub use crate::collision::*;
    pub use crate::components::*;
    pub use crate::core::*;
    pub use crate::event::*;
    pub use crate::input::*;
    pub use crate::math::*;
    pub use crate::render::*;
    pub use crate::resources::*;
    pub use crate::runtime::*;
    pub use crate::ui::*;
    pub use macros::*;
}

use bincode::config::standard;
use std::time::Duration;
use tokio::time::sleep;

pub fn serialize_bytes<T: bincode::Encode>(item: &T) -> Vec<u8> {
    bincode::encode_to_vec(item, standard()).expect("Falha crítica ao serializar bytes")
}

pub fn deserialize_bytes<T: bincode::Decode<()>>(bytes: &[u8]) -> Option<T> {
    match bincode::decode_from_slice(bytes, standard()) {
        Ok((item, _bytes_read)) => Some(item),
        Err(_e) => None,
    }
}

pub async fn sleep_tokio(secs: f32) {
    sleep(Duration::from_secs_f32(secs)).await;
}
