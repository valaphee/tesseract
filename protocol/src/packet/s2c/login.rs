use crate::types::GameProfile;
use crate::{types::VarInt, Decode, Encode};

#[derive(Encode, Decode)]
pub enum LoginPacket {
    LoginDisconnect(LoginDisconnectPacket),
    Hello(HelloPacket),
    GameProfile(GameProfilePacket),
    LoginCompression(LoginCompressionPacket),
    CustomQuery(CustomQueryPacket),
}

#[derive(Encode, Decode)]
pub struct LoginDisconnectPacket {
    pub reason: String,
}

#[derive(Encode, Decode)]
pub struct HelloPacket {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Encode, Decode)]
pub struct GameProfilePacket {
    pub game_profile: GameProfile,
}

#[derive(Encode, Decode)]
pub struct LoginCompressionPacket {
    pub compression_threshold: VarInt,
}

#[derive(Encode, Decode)]
pub struct CustomQueryPacket {
    pub transaction_id: VarInt,
    pub identifier: String,
    pub data: (/*TODO*/),
}
