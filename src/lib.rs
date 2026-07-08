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
use bincode::config::standard;
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
