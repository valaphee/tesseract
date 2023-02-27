#![feature(array_try_from_fn)]

pub use tesseract_protocol_derive::{Decode, Encode};

pub mod packet;
pub mod types;

trait Encode {
    fn encode<W: std::io::Write>(&self, output: &mut W) -> anyhow::Result<()>;
}

trait Decode: Sized {
    fn decode<R: std::io::Read>(input: &mut R) -> anyhow::Result<Self>;
}
