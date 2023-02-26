use crate::{Decode, Encode, VarInt};

#[derive(Encode, Decode)]
pub enum ClientboundPacket {
    LoginDisconnect {
        reason: String,
    },
    Hello {
        server_id: String,
        public_key: Vec<u8>,
        nonce: Vec<u8>,
    },
    GameProfile,
    LoginCompression {
        compression_threshold: VarInt,
    },
    CustomQuery {
        transaction_id: VarInt,
        identifier: String,
    },
}

#[derive(Encode, Decode)]
pub enum ServerboundPacket {
    Hello { name: String },
    Key { key: Vec<u8>, nonce: Vec<u8> },
    CustomQuery { transaction_id: VarInt },
}
