use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum GamePacket<T: Encode + Decode<()>> {
    Unrealiable { id: u32, payload: u32 },
    Realiable { id: u32, payload: T },
}
