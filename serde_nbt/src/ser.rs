use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::error::{Error, Result};
use crate::TagType;

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: serde::ser::Serialize,
{
    let mut ser = Serializer {
        data: vec![0x0A, 0x00, 0x00],

        last_type: TagType::default(),
    };
    value.serialize(&mut ser)?;
    Ok(ser.data)
}

struct Serializer {
    data: Vec<u8>,

    last_type: TagType,
}

impl<'ser> serde::ser::Serializer for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeSeq<'ser>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.data.write_i8(match v {
            true => 1,
            false => 0,
        })?;
        self.last_type = TagType::Byte;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.data.write_i8(v)?;
        self.last_type = TagType::Byte;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.data.write_i16::<BigEndian>(v)?;
        self.last_type = TagType::Short;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.data.write_i32::<BigEndian>(v)?;
        self.last_type = TagType::Int;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.data.write_i64::<BigEndian>(v)?;
        self.last_type = TagType::Long;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.data.write_i8(v as i8)?;
        self.last_type = TagType::Byte;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.data.write_i16::<BigEndian>(v as i16)?;
        self.last_type = TagType::Short;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.data.write_i32::<BigEndian>(v as i32)?;
        self.last_type = TagType::Int;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.data.write_i64::<BigEndian>(v as i64)?;
        self.last_type = TagType::Long;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.data.write_f32::<BigEndian>(v)?;
        self.last_type = TagType::Float;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.data.write_f64::<BigEndian>(v)?;
        self.last_type = TagType::Double;
        Ok(())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let bytes = v.as_bytes();
        self.data.write_i16::<BigEndian>(bytes.len() as i16)?;
        self.data.write(bytes)?;
        self.last_type = TagType::String;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.data.write_i32::<BigEndian>(v.len() as i32)?;
        self.data.write(v)?;
        self.last_type = TagType::ByteArray;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        // this is only needed for support writing optional fields
        // type TagType::End is reserved, and is used here for marking none
        self.last_type = TagType::End;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        // this is only needed for support writing optional fields
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        // Write empty header, as type and length is not known yet
        let header_offset = self.data.len();
        self.data.write_i8(0)?;
        self.data.write_i32::<BigEndian>(0)?;

        // Reset type_, so that an empty list is of type TagType::End
        self.last_type = TagType::End;
        Ok(SerializeSeq {
            header_offset,
            count: 0,
            de: self,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!()
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!()
    }
}

struct SerializeSeq<'ser> {
    de: &'ser mut Serializer,

    header_offset: usize,
    count: i32,
}

impl<'ser> serde::ser::SerializeSeq for SerializeSeq<'ser> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        self.count += 1;
        value.serialize(&mut *self.de)
    }

    fn end(self) -> Result<Self::Ok> {
        // Override empty header
        let mut header = &mut self.de.data[self.header_offset..self.header_offset + 5];
        header.write_i8(self.de.last_type.into())?;
        header.write_i32::<BigEndian>(self.count)?;

        // Set last type to TagType::List
        self.de.last_type = TagType::List;
        Ok(())
    }
}

impl<'ser> serde::ser::SerializeTuple for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!()
    }
}

impl<'ser> serde::ser::SerializeTupleStruct for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!()
    }
}

impl<'ser> serde::ser::SerializeTupleVariant for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!()
    }
}

impl<'ser> serde::ser::SerializeMap for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!()
    }
}

impl<'ser> serde::ser::SerializeStruct for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        let header_offset = self.data.len();
        value.serialize(&mut **self)?;

        // Write field, or omit if the field is not existent, see serialize_none
        if self.last_type != TagType::End {
            let mut header = Vec::new();
            header.write_i8(self.last_type.into())?;
            let key_bytes = key.as_bytes();
            header.write_i16::<BigEndian>(key_bytes.len() as i16)?;
            header.write(key_bytes)?;
            self.data.splice(header_offset..header_offset, header);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.data.write_i8(TagType::End.into())?;
        self.last_type = TagType::Compound;
        Ok(())
    }
}

impl<'ser> serde::ser::SerializeStructVariant for &'ser mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!()
    }
}
