mod client;
mod server;

use bincode::{Decode, Encode};
pub use client::*;
pub use server::*;

#[derive(Encode, Decode)]
pub enum NetworkEvent {
    Connected,
    Disconnected,
}
