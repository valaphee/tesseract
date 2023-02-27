use crate::{types::VarInt, Decode, Encode};
use uuid::Uuid;

#[derive(Encode, Decode)]
pub enum LoginPacket {
    Hello(HelloPacket),
    Key(KeyPacket),
    CustomQuery(CustomQueryPacket),
}

#[derive(Encode, Decode)]
pub struct HelloPacket {
    pub name: String,
    pub uuid: Option<Uuid>,
}

#[derive(Encode, Decode)]
pub struct KeyPacket {
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Encode, Decode)]
pub struct CustomQueryPacket {
    pub transaction_id: VarInt,
    pub data: (/*TODO*/),
}
