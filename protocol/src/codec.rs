use std::io::Read;

use aes::{
    cipher::{inout::InOutBuf, BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Aes128,
};
use bytes::{Buf, BufMut, BytesMut};
use cfb8::{Decryptor, Encryptor};
use flate2::read::{ZlibDecoder, ZlibEncoder};
pub use flate2::Compression;
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    types::{VarI21, VarI32},
    Decode, Encode, Error, Result,
};

#[derive(Default)]
pub struct Codec {
    encryptor: Option<Encryptor<Aes128>>,
    decryptor: Option<Decryptor<Aes128>>,
    decrypted_bytes: usize,

    compression: Compression,
    compression_threshold: Option<u16>,
}

impl Codec {
    pub fn enable_encryption(&mut self, key: &[u8]) {
        self.encryptor = Some(Encryptor::new_from_slices(key, key).unwrap());
        self.decryptor = Some(Decryptor::new_from_slices(key, key).unwrap());
        self.decrypted_bytes = 0;
    }

    pub fn enable_compression(&mut self, compression: Compression, compression_threshold: u16) {
        assert!(compression_threshold <= 16384);
        self.compression = compression;
        self.compression_threshold = Some(compression_threshold);
    }
}

impl Encoder<&[u8]> for Codec {
    type Error = Error;

    fn encode(&mut self, item: &[u8], dst: &mut BytesMut) -> Result<()> {
        let data_length_offset = dst.len();
        dst.put_bytes(0, 3);
        let data_offset = dst.len();
        dst.put_slice(item);
        let mut data_length = dst.len() - data_offset;

        if let Some(compression_threshold) = self.compression_threshold {
            if data_length > compression_threshold as usize {
                let mut compressed_data = Vec::new();
                ZlibEncoder::new(&dst[data_offset..], self.compression)
                    .read_to_end(&mut compressed_data)
                    .unwrap();

                dst.truncate(data_length_offset);
                let mut writer = dst.writer();
                let data_length_varint = VarI32(data_length as i32);
                VarI32((data_length_varint.len() + compressed_data.len()) as i32)
                    .encode(&mut writer)?;
                data_length_varint.encode(&mut writer)?;
                dst.extend_from_slice(&compressed_data);
            } else {
                data_length += 1;

                // This will limit the maximum compression threshold to 16384 (2 VarInt bytes)
                // as the third VarInt byte has to be kept zero to indicate no
                // compression.
                let data_length_data = &mut dst[data_length_offset..data_offset];
                data_length_data[0] = (data_length & 0x7F) as u8 | 0x80;
                data_length_data[1] = (data_length >> 7 & 0x7F) as u8;
            }
        } else {
            let data_length_data = &mut dst[data_length_offset..data_offset];
            data_length_data[0] = (data_length & 0x7F) as u8 | 0x80;
            data_length_data[1] = (data_length >> 7 & 0x7F) as u8 | 0x80;
            data_length_data[2] = (data_length >> 14 & 0x7F) as u8;
        }

        // Encrypt written bytes
        if let Some(encryptor) = &mut self.encryptor {
            encryptor.encrypt_blocks_inout_mut(
                InOutBuf::from(&mut dst[data_length_offset..])
                    .into_chunks()
                    .0,
            );
        }

        Ok(())
    }
}

impl Decoder for Codec {
    type Item = Vec<u8>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        // Decrypt all not yet decrypted bytes
        if let Some(decryptor) = &mut self.decryptor {
            decryptor.decrypt_blocks_inout_mut(
                InOutBuf::from(&mut src[self.decrypted_bytes..])
                    .into_chunks()
                    .0,
            );
            self.decrypted_bytes = src.len();
        }

        let mut data = &src[..];
        match VarI21::decode(&mut data) {
            Ok(data_length) => {
                if data.len() >= data_length.0 as usize {
                    data = &data[..data_length.0 as usize];

                    let data = if self.compression_threshold.is_some() {
                        let decompressed_data_length = VarI32::decode(&mut data)?;
                        if decompressed_data_length.0 != 0 {
                            let mut decompressed_data =
                                Vec::with_capacity(decompressed_data_length.0 as usize);
                            ZlibDecoder::new(data)
                                .read_to_end(&mut decompressed_data)
                                .unwrap();
                            decompressed_data
                        } else {
                            data.to_vec()
                        }
                    } else {
                        data.to_vec()
                    };

                    // Advance, and correct decrypted bytes
                    src.advance(data_length.len() + data_length.0 as usize);
                    if self.decryptor.is_some() {
                        self.decrypted_bytes = src.len()
                    }

                    Ok(Some(data))
                } else {
                    Ok(None)
                }
            }
            Err(error) => {
                if data.len() >= 3 {
                    Err(error)
                } else {
                    Ok(None)
                }
            }
        }
    }
}
