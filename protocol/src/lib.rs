#![feature(array_try_from_fn)]

extern crate core;

use std::marker::PhantomData;
use bytes::{Buf, BytesMut};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
pub use tesseract_protocol_derive::{Decode, Encode};
use crate::types::VarInt;

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

pub struct Codec<T> {
    _phantom: PhantomData<T>
}

impl<T> Codec<T> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<T> Encoder<T> for Codec<T>
where
    T: Encode
{
    type Error = Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<()> {
        item.encode(&mut &mut dst[..])
    }
}

impl<T> Decoder for Codec<T>
where
    T: Decode<'static>,
{
    type Item = T;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let mut data = &src[..];
        match VarInt::decode(&mut data) {
            Ok(length) => if src.len() >= length.0 as usize {
                data = &data[..length.0 as usize];
                let packet = T::decode(unsafe { std::mem::transmute(&mut data) })?;
                src.advance(data.as_ptr() as usize - src.as_ptr() as usize);
                Ok(Some(packet))
            } else {
                Ok(None)
            }
            Err(_) => Ok(None)
        }
    }
}
