use crate::types::{GameProfile, TrailingBytes};
use crate::{types::VarInt, Decode, Encode};

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
    GameProfile {
        game_profile: GameProfile,
    },
    LoginCompression {
        compression_threshold: VarInt,
    },
    CustomQuery {
        transaction_id: VarInt,
        identifier: String,
        data: TrailingBytes,
    },
}
