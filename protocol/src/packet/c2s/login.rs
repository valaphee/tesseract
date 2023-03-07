use uuid::Uuid;

use crate::{
    types::{TrailingBytes, VarInt},
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum LoginPacket {
    Hello {
        name: String,
        uuid: Option<Uuid>,
    },
    Key {
        key: Vec<u8>,
        nonce: Vec<u8>,
    },
    CustomQuery {
        transaction_id: VarInt,
        data: TrailingBytes,
    },
}
