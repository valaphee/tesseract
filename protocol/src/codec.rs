use std::marker::PhantomData;

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{types::VarInt, Decode, Encode, Error, Result};

pub struct Codec<I, O> {
    _phantom: PhantomData<(I, O)>,
}

impl<I, O> Codec<I, O> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<I, O> Encoder<I> for Codec<I, O>
where
    I: Encode,
{
    type Error = Error;

    fn encode(&mut self, item: I, dst: &mut BytesMut) -> Result<()> {
        let mut data = Vec::new();
        item.encode(&mut data)?;

        let mut frame_length_data = Vec::new();
        VarInt(data.len() as i32).encode(&mut frame_length_data)?;

        dst.put_slice(&frame_length_data);
        dst.put_slice(&data);
        Ok(())
    }
}

impl<I, O> Decoder for Codec<I, O>
where
    O: Decode<'static>,
{
    type Item = O;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let mut data = &src[..];
        match VarInt::decode(&mut data) {
            Ok(frame_length) => {
                if src.len() >= frame_length.0 as usize {
                    data = &data[..frame_length.0 as usize];
                    let packet = O::decode(unsafe { std::mem::transmute(&mut data) })?;
                    src.advance(data.as_ptr() as usize - src.as_ptr() as usize);
                    Ok(Some(packet))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }
}
