use byteorder::{BigEndian, ReadBytesExt};
use serde::forward_to_deserialize_any;

use crate::{
    error::{Error, Result},
    TagType,
};

pub fn from_slice<'a, T>(input: &mut &'a [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(input)?;
    let value = T::deserialize(&mut deserializer);
    *input = deserializer.data;
    value
}

struct Deserializer<'de> {
    data: &'de [u8],

    name: bool,
    current_type: TagType,
}

impl<'de> Deserializer<'de> {
    fn from_slice(input: &'de [u8]) -> Result<Self> {
        let mut self_ = Self {
            data: input,

            name: false,
            current_type: TagType::default(),
        };
        let type_ = TagType::try_from(self_.data.read_i8()?).unwrap();
        if type_ != TagType::End {
            let name_length = self_.data.read_i16::<BigEndian>()?;
            let (_, data) = self_.data.split_at(name_length as usize);
            self_.data = data;
        }
        self_.current_type = type_;
        Ok(self_)
    }
}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf unit unit_struct
        newtype_struct seq tuple tuple_struct map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.name {
            self.name = false;

            let length = self.data.read_i16::<BigEndian>()?;
            let (bytes, data) = self.data.split_at(length as usize);
            self.data = data;
            visitor.visit_str(std::str::from_utf8(bytes).unwrap())
        } else {
            match self.current_type {
                TagType::End => visitor.visit_unit(),
                TagType::Byte => visitor.visit_i8(self.data.read_i8()?),
                TagType::Short => visitor.visit_i16(self.data.read_i16::<BigEndian>()?),
                TagType::Int => visitor.visit_i32(self.data.read_i32::<BigEndian>()?),
                TagType::Long => visitor.visit_i64(self.data.read_i64::<BigEndian>()?),
                TagType::Float => visitor.visit_f32(self.data.read_f32::<BigEndian>()?),
                TagType::Double => visitor.visit_f64(self.data.read_f64::<BigEndian>()?),
                TagType::ByteArray => visitor.visit_seq(SeqAccess {
                    type_: TagType::Byte,
                    count: self.data.read_i32::<BigEndian>()? as u32,
                    de: self,
                }),
                TagType::String => {
                    let length = self.data.read_i16::<BigEndian>()?;
                    let (bytes, data) = self.data.split_at(length as usize);
                    self.data = data;
                    visitor.visit_str(std::str::from_utf8(bytes).unwrap())
                }
                TagType::List => visitor.visit_seq(SeqAccess {
                    type_: TagType::try_from(self.data.read_i8()?).unwrap(),
                    count: self.data.read_i32::<BigEndian>()? as u32,
                    de: self,
                }),
                TagType::Compound => visitor.visit_map(self),
                TagType::IntArray => visitor.visit_seq(SeqAccess {
                    type_: TagType::Int,
                    count: self.data.read_i32::<BigEndian>()? as u32,
                    de: self,
                }),
                TagType::LongArray => visitor.visit_seq(SeqAccess {
                    type_: TagType::Long,
                    count: self.data.read_i32::<BigEndian>()? as u32,
                    de: self,
                }),
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.name {
            self.name = false;

            let length = self.data.read_i16::<BigEndian>()?;
            let (bytes, data) = self.data.split_at(length as usize);
            self.data = data;
            visitor.visit_str(std::str::from_utf8(bytes).unwrap())
        } else {
            match self.current_type {
                TagType::Byte => visitor.visit_bool(match self.data.read_i8()? {
                    0 => false,
                    1 => true,
                    _ => unimplemented!()
                }),
                _ => unimplemented!()
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        // this is only needed for support reading optional fields
        visitor.visit_some(self)
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }
}

struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,

    type_: TagType,
    count: u32,
}

impl<'a, 'de> serde::de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.count == 0 {
            return Ok(None);
        }
        self.count -= 1;

        self.de.current_type = self.type_;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count as usize)
    }
}

impl<'a, 'de> serde::de::MapAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.current_type = TagType::try_from(self.data.read_i8()?).unwrap();
        if !matches!(self.current_type, TagType::End) {
            self.name = true;
            seed.deserialize(&mut **self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut **self)
    }
}
