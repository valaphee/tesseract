use crate::error::{Error, Result};
use crate::TagType;
use byteorder::{BigEndian, ReadBytesExt};

pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    T::deserialize(&mut Deserializer::from_slice(v)?)
}

struct Deserializer<'de> {
    data: &'de [u8],

    key: bool,
    type_: TagType,
}

impl<'de> Deserializer<'de> {
    fn from_slice(data: &'de [u8]) -> Result<Self> {
        let mut self_ = Self {
            data,

            key: false,
            type_: TagType::default(),
        };
        let type_ = TagType::try_from(self_.data.read_i8()?).unwrap();
        let key_length = self_.data.read_i16::<BigEndian>()?;
        let (_, data) = self_.data.split_at(key_length as usize);
        self_.data = data;
        self_.type_ = type_;
        Ok(self_)
    }
}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.key {
            self.key = false;

            let length = self.data.read_i16::<BigEndian>()?;
            let (bytes, data) = self.data.split_at(length as usize);
            self.data = data;
            visitor.visit_str(std::str::from_utf8(bytes).unwrap())
        } else {
            match self.type_ {
                TagType::End => unreachable!(),
                TagType::Byte => visitor.visit_i8(self.data.read_i8()?),
                TagType::Short => visitor.visit_i16(self.data.read_i16::<BigEndian>()?),
                TagType::Int => visitor.visit_i32(self.data.read_i32::<BigEndian>()?),
                TagType::Long => visitor.visit_i64(self.data.read_i64::<BigEndian>()?),
                TagType::Float => visitor.visit_f32(self.data.read_f32::<BigEndian>()?),
                TagType::Double => visitor.visit_f64(self.data.read_f64::<BigEndian>()?),
                /*TagType::ByteArray => {
                    let length = self.data.read_i32::<BigEndian>()?;
                    let (bytes, data) = self.data.split_at(length as usize);
                    self.data = data;
                    visitor.visit_bytes(bytes)
                },*/
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
                TagType::Compound => visitor.visit_map(MapAccess { de: self }),
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
        self.deserialize_any(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        // this is only needed for support reading optional fields
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(MapAccess { de: self })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
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
        self.de.type_ = self.type_;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count as usize)
    }
}

struct MapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> serde::de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.de.type_ = TagType::try_from(self.de.data.read_i8()?).unwrap();
        if !matches!(self.de.type_, TagType::End) {
            self.de.key = true;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}
