use crate::{Decode, Encode, VarInt};

#[derive(Encode, Decode)]
pub enum ServerboundPacket {
    Intention {
        protocol_version: VarInt,
        host_name: String,
        port: u16,
        intention: Intention,
    },
}

#[derive(Encode, Decode)]
pub enum Intention {
    Game,
    Status,
    Login,
}
