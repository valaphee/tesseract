use crate::{
    types::{Component, Json, TrailingBytes, User, VarI32},
    Decode, Encode,
};

#[derive(Encode, Decode, Clone, Debug)]
pub enum LoginPacket {
    LoginDisconnect {
        reason: Json<Component>,
    },
    Hello {
        server_id: String,
        public_key: Vec<u8>,
        nonce: Vec<u8>,
    },
    GameProfile(User),
    LoginCompression {
        #[using(VarI32)]
        compression_threshold: i32,
    },
    CustomQuery {
        #[using(VarI32)]
        transaction_id: i32,
        identifier: String,
        data: TrailingBytes<{ 1 << 20 }>,
    },
}
