#![feature(array_try_from_fn)]

extern crate core;

use thiserror::Error;

pub use tesseract_protocol_derive::{Decode, Encode};

pub mod bit_storage;
pub mod codec;
pub mod packet;
pub mod types;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("UTF8 error")]
    Utf8Str(#[from] std::str::Utf8Error),
    #[error("UTF8 error")]
    Utf8String(#[from] std::string::FromUtf8Error),
    #[error("Json error")]
    Json(#[from] serde_json::Error),
    #[error("Nbt error")]
    Nbt(#[from] tesseract_serde_nbt::error::Error),

    #[error("VarInt wider than {0}-bit")]
    VarIntTooWide(u8),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Encode {
    fn encode<W: std::io::Write>(&self, output: &mut W) -> Result<()>;
}

pub trait Decode<'a>: Sized {
    fn decode(input: &mut &'a [u8]) -> Result<Self>;
}
