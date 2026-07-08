mod client;
mod server;

use bincode::{Decode, Encode};
pub use client::*;
pub use server::*;

use crate::serialize_bytes;

#[derive(Encode, Decode)]
pub enum NetworkEvent {
    Connected,
    Disconnected,
}
impl NetworkEvent {
    pub fn into_bytes(&self) -> Vec<u8> {
        serialize_bytes(self)
    }
}
