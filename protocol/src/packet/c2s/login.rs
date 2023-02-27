use crate::{types::VarInt, Decode, Encode};

#[derive(Encode, Decode)]
pub enum LoginPacket {
    Hello(HelloPacket),
    Key(KeyPacket),
    CustomQuery(CustomQueryPacket),
}

#[derive(Encode, Decode)]
pub struct HelloPacket {
    name: String,
}

#[derive(Encode, Decode)]
pub struct KeyPacket {
    key: Vec<u8>,
    nonce: Vec<u8>,
}

#[derive(Encode, Decode)]
pub struct CustomQueryPacket {
    transaction_id: VarInt,
}
