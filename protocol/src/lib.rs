pub mod packet;

pub use tesseract_protocol_derive::{Decode, Encode};

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{arch, io};
use uuid::Uuid;

trait Encode {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()>;
}

trait Decode: Sized {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self>;
}

impl Encode for bool {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        if *self { 1u8 } else { 0u8 }.encode(output)
    }
}

impl Decode for bool {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        Ok(match u8::decode(input)? {
            0 => false,
            1 => true,
            _ => todo!(),
        })
    }
}

impl Encode for u8 {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(*self)?;
        Ok(())
    }
}

impl Decode for u8 {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        Ok(input.read_u8()?)
    }
}

impl Encode for u16 {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        output.write_u16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for u16 {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        Ok(input.read_u16::<BigEndian>()?)
    }
}

impl Encode for u64 {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        output.write_u64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for u64 {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        Ok(input.read_u64::<BigEndian>()?)
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        match self {
            None => false.encode(output),
            Some(value) => {
                true.encode(output)?;
                value.encode(output)
            }
        }
    }
}

impl<T> Decode for Option<T>
where
    T: Decode,
{
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        Ok(match bool::decode(input)? {
            true => Some(T::decode(input)?),
            false => None,
        })
    }
}

pub struct VarInt(pub i32);

impl Encode for VarInt {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        let data = unsafe { arch::x86_64::_pdep_u64(self.0 as u64, 0x0000000000037F7F) };
        let length = 8 - ((data.leading_zeros() - 1) >> 3);
        let encoded =
            data | (0x8080808080808080 & (0xFFFFFFFFFFFFFFFF >> (((8 - length + 1) << 3) - 1)));
        output.write_all(unsafe { encoded.to_le_bytes().get_unchecked(..length as usize) })?;
        Ok(())
    }
}

impl Decode for VarInt {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        let mut value = 0;
        let mut shift = 0;
        while shift <= 35 {
            let head = input.read_u8()?;
            value |= (head as i32 & 0b01111111) << shift;
            if head & 0b10000000 == 0 {
                return Ok(VarInt(value));
            }
            shift += 7;
        }
        todo!()
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        VarInt(self.len() as i32).encode(output)?;
        for item in self.iter() {
            item.encode(output)?;
        }
        Ok(())
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        let length = VarInt::decode(input)?.0 as usize;
        let mut value = Vec::with_capacity(length);
        for _ in 0..length {
            value.push(T::decode(input)?);
        }
        Ok(value)
    }
}

impl Encode for String {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        self.as_bytes().to_vec().encode(output)?;
        Ok(())
    }
}

impl Decode for String {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        Ok(String::from_utf8(Vec::<u8>::decode(input)?)?)
    }
}

impl Encode for Uuid {
    fn encode<W: io::Write>(&self, output: &mut W) -> Result<()> {
        output.write_u128::<BigEndian>(self.as_u128())?;
        Ok(())
    }
}

impl Decode for Uuid {
    fn decode<R: io::Read>(input: &mut R) -> Result<Self> {
        Ok(Uuid::from_u128(input.read_u128::<BigEndian>()?))
    }
}
