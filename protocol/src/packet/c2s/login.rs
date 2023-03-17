use uuid::Uuid;

use crate::{
    types::{TrailingBytes, VarI32},
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
        transaction_id: VarI32,
        data: TrailingBytes<{ 1 << 20 }>,
    },
}
