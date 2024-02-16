use serde::Serialize;

use crate::vtable::*;
use crate::{error::Error, traits::DocBuf, Result};

pub struct DocBufSerializer {
    pub vtable: &'static VTable<'static>,
    pub output: Vec<u8>,
    pub current_struct_index: u8,
    pub current_field_index: u8,
}

impl DocBufSerializer {
    pub fn new(vtable: &'static VTable) -> Self {
        Self {
            vtable,
            output: Vec::new(),
            current_struct_index: 0,
            current_field_index: 0,
        }
    }

    // Encode the data into the data map
    pub fn encode_data(&mut self, data: impl AsRef<[u8]>) -> Result<()> {
        self.vtable
            .struct_by_index(&self.current_struct_index)?
            .field_by_index(&self.current_field_index)?
            .encode(data.as_ref(), &mut self.output)?;

        Ok(())
    }
}

pub fn to_docbuf<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + DocBuf + std::fmt::Debug,
{
    let mut serializer = DocBufSerializer::new(T::vtable()?);

    value.serialize(&mut serializer)?;

    Ok(serializer.output)
}

impl<'a> serde::ser::Serializer for &'a mut DocBufSerializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self> {
        // Check the vtable for the entry
        if let Some(entry) = self.vtable.structs.get(name.as_bytes()) {
            // Set current index
            self.current_struct_index = entry.struct_index;

            // Check if entry fields match the length of the fields from the serializer
            if entry.num_fields != len as u8 {
                return Err(Error::Serde(format!(
                    "Number of fields in struct {} does not match the vtable",
                    name
                )));
            }
        } else {
            return Err(Error::Serde(format!(
                "Struct {} not found in the vtable",
                name
            )));
        }

        Ok(self)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        unimplemented!("serialize_bool")
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        unimplemented!("serialize_bytes")
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        unimplemented!("serialize_char")
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        unimplemented!("serialize_f32")
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        unimplemented!("serialize_f64")
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        unimplemented!("serialize_i8")
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        unimplemented!("serialize_i16")
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        unimplemented!("serialize_i32")
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        unimplemented!("serialize_i64")
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok> {
        unimplemented!("serialize_i128")
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.encode_data(v.as_bytes())?;

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.encode_data(v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.encode_data(v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        unimplemented!("serialize_u32")
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        unimplemented!("serialize_u64")
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok> {
        unimplemented!("serialize_u128")
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        unimplemented!("serialize_none")
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!("serialize_some")
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        unimplemented!("serialize_unit")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        unimplemented!("serialize_unit_struct")
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        unimplemented!("serialize_unit_variant")
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!("serialize_newtype_struct")
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!("serialize_newtype_variant")
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unimplemented!("serialize_seq")
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!("serialize_tuple")
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!("serialize_tuple_struct")
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!("serialize_tuple_variant")
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!("serialize_map")
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!("serialize_struct_variant")
    }
}

impl<'a> serde::ser::SerializeSeq for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_element")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}

impl<'a> serde::ser::SerializeTuple for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_element")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}

impl<'a> serde::ser::SerializeTupleStruct for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_field")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}

impl<'a> serde::ser::SerializeTupleVariant for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_field")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}

impl<'a> serde::ser::SerializeMap for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_key")
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_value")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}

impl<'a> serde::ser::SerializeStruct for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.current_field_index = self
            .vtable
            .struct_by_index(&self.current_struct_index)?
            .field_index_from_name(key)?;

        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Update the current index
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_field")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}
