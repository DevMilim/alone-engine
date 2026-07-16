use bincode::{Decode, Encode};

mod client;
mod server;

pub use client::*;
pub use server::*;

#[derive(Encode, Decode)]
pub enum NetworkEvent {
    Connected,
    Disconnected,
    ConnectFailed,
}

pub enum NetworkError {
    NotAClient,
    NotAServer,
    ChannelFull,
    Disconnected,
}
