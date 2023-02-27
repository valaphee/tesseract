#![feature(array_try_from_fn)]

pub use tesseract_protocol_derive::{Decode, Encode};

pub mod packet;
pub mod types;

pub trait Encode {
    fn encode<W: std::io::Write>(&self, output: &mut W) -> anyhow::Result<()>;
}

pub trait Decode<'a>: Sized {
    fn decode(input: &mut &'a [u8]) -> anyhow::Result<Self>;
}
