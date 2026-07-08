mod audio;
mod collision;
mod components;
mod core;
mod event;
mod input;
mod math;
mod network;
mod render;
mod resources;
mod runtime;

pub use audio::*;
use bincode::{Decode, Encode, decode_from_slice, encode_to_vec};
pub use collision::*;
pub use components::*;
pub use core::*;
pub use event::*;
pub use input::*;
pub use macros::*;
pub use math::*;
pub use network::*;
pub use render::*;
pub use resources::*;
pub use runtime::*;
use std::time::Duration;
use tokio::time::sleep;

pub fn serialize_bytes<T: Encode>(value: &T) -> Vec<u8> {
    encode_to_vec(value, bincode::config::standard()).unwrap()
}

pub fn deserialize_bytes<T: Decode<()>>(bytes: &[u8]) -> Option<T> {
    decode_from_slice(bytes, bincode::config::standard())
        .ok()?
        .0
}

pub async fn sleep_tokio(secs: f32) {
    sleep(Duration::from_secs_f32(secs)).await;
}
