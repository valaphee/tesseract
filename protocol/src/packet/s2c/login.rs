use crate::{
    types::{TrailingBytes, User, VarI32},
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
        compression_threshold: VarI32,
    },
    CustomQuery {
        transaction_id: VarI32,
        identifier: String,
        data: TrailingBytes,
    },
}
