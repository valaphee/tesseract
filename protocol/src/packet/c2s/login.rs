use uuid::Uuid;

use crate::{
    types::{TrailingBytes, VarI32},
    Decode, Encode,
};

#[derive(Encode, Decode, Clone, Debug)]
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
        transaction_id: VarI32,
        data: TrailingBytes<{ 1 << 20 }>,
    },
}
