use std::io::Read;
use std::marker::PhantomData;

use bytes::{Buf, BufMut, BytesMut};
use flate2::Compression;
use flate2::read::{ZlibDecoder, ZlibEncoder};
use tokio_util::codec::{Decoder, Encoder};

use crate::{types::VarInt, Decode, Encode, Error, Result};

pub struct Codec<I, O> {
    pub compression: Compression,
    pub compression_threshold: Option<usize>,

    _phantom: PhantomData<(I, O)>,
}

impl<I, O> Codec<I, O> {
    pub fn new() -> Self {
        Self {
            compression: Compression::default(),
            compression_threshold: None,

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
        fn set_varint14(data: &mut [u8], value: usize) {
            data[0] = (value & 0x7F) as u8 | 0x80;
            data[1] = (value >> 7 & 0x7F) as u8;
        }

        fn set_varint21(data: &mut [u8], value: usize) {
            data[0] = (value & 0x7F) as u8 | 0x80;
            data[1] = (value >> 7 & 0x7F) as u8 | 0x80;
            data[2] = (value >> 14 & 0x7F) as u8;
        }

        let data_length_offset = dst.len();
        dst.put_bytes(0, 3);
        let data_offset = dst.len();
        item.encode(&mut dst.writer())?;
        let mut data_length = dst.len() - data_offset;

        if let Some(compression_threshold) = self.compression_threshold {
            if data_length > compression_threshold {
                let mut compressed_data = Vec::new();
                ZlibEncoder::new(&dst[data_offset..], self.compression).read_to_end(&mut compressed_data).unwrap();

                dst.truncate(data_length_offset);
                let mut writer = dst.writer();
                let data_length_varint = VarInt(data_length as i32);
                VarInt((data_length_varint.len() + compressed_data.len()) as i32).encode(&mut writer)?;
                data_length_varint.encode(&mut writer)?;
                dst.extend_from_slice(&mut compressed_data);
            } else {
                let data_length_data = &mut dst[data_length_offset..data_offset];
                data_length += 1;
                data_length_data[0] = (data_length & 0x7F) as u8 | 0x80;
                data_length_data[1] = (data_length >> 7 & 0x7F) as u8;
            }
        } else {
            let data_length_data = &mut dst[data_length_offset..data_offset];
            data_length_data[0] = (data_length & 0x7F) as u8 | 0x80;
            data_length_data[1] = (data_length >> 7 & 0x7F) as u8 | 0x80;
            data_length_data[2] = (data_length >> 14 & 0x7F) as u8;
        }

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
            Ok(data_length) => {
                if src.len() >= data_length.0 as usize {
                    let data_length_length = data.as_ptr() as usize - src.as_ptr() as usize;

                    data = &data[..data_length.0 as usize];

                    let packet = if self.compression_threshold.is_some() {
                        let decompressed_data_length = VarInt::decode(&mut data)?.0 as usize;
                        if decompressed_data_length != 0 {
                            let mut decompressed_data = Vec::with_capacity(decompressed_data_length);
                            ZlibDecoder::new(data).take(decompressed_data_length as u64).read_to_end(&mut decompressed_data).unwrap();
                            O::decode(unsafe { std::mem::transmute(&mut decompressed_data) })?
                        } else {
                            O::decode(unsafe { std::mem::transmute(&mut data) })?
                        }
                    } else {
                        O::decode(unsafe { std::mem::transmute(&mut data) })?
                    };

                    src.advance(data_length_length + data_length.0 as usize);

                    Ok(Some(packet))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }
}
