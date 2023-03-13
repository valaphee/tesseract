use crate::{
    types::{TrailingBytes, User, VarInt32},
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum LoginPacket {
    LoginDisconnect {
        reason: String,
    },
    Hello {
        server_id: String,
        public_key: Vec<u8>,
        nonce: Vec<u8>,
    },
    GameProfile(User),
    LoginCompression {
        compression_threshold: VarInt32,
    },
    CustomQuery {
        transaction_id: VarInt32,
        identifier: String,
        data: TrailingBytes,
    },
}
