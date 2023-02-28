#![feature(array_try_from_fn)]

extern crate core;

use bytes::{Buf, BufMut, BytesMut};
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

pub struct Codec;

impl Encoder<Vec<u8>> for Codec {
    type Error = Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<()> {
        let mut length = Vec::new();
        VarInt(item.len() as i32).encode(&mut length)?;
        dst.reserve(length.len() + item.len());
        dst.put_slice(&length);
        dst.put_slice(&item);
        Ok(())
    }
}

impl Decoder for Codec {
    type Item = Vec<u8>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let mut data = &src[..];
        match VarInt::decode(&mut data) {
            Ok(frame_length) => {
                if src.len() >= frame_length.0 as usize {
                    let frame = data[..frame_length.0 as usize].to_vec();
                    src.advance(
                        (data.as_ptr() as usize - src.as_ptr() as usize) + frame_length.0 as usize,
                    );
                    Ok(Some(frame))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }
}
