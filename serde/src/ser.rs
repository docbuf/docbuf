use crate::{error, Result};

use serde::{Serializer, Serialize};



pub struct DocBufSerializer {
    pub output: Vec<u8>,
}

pub fn to_docbuf<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = DocBufSerializer {
        output: Vec::new(),
    };

    value.serialize(&mut serializer)?;

    Ok(serializer.output)
}

impl<'a> serde::ser::Serializer for &'a mut DocBufSerializer {
    type Ok = ();

    type Error = error::Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;


    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self> {
        println!("serialize_struct");

        println!("name: {:?}", name);
        println!("len: {:?}", len);
        
        unimplemented!("serialize_struct")
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        unimplemented!("serialize_bool")
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        unimplemented!("serialize_bytes")
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        unimplemented!("serialize_char")
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        unimplemented!("serialize_f32")
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        unimplemented!("serialize_f64")
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        unimplemented!("serialize_i8")
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        unimplemented!("serialize_i16")
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        unimplemented!("serialize_i32")
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        unimplemented!("serialize_i64")
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        unimplemented!("serialize_i128")
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        unimplemented!("serialize_str")
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        unimplemented!("serialize_u8")
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        unimplemented!("serialize_u16")
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        unimplemented!("serialize_u32")
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        unimplemented!("serialize_u64")
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        unimplemented!("serialize_u128")
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        unimplemented!("serialize_none")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!("serialize_some")
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        unimplemented!("serialize_unit")
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        unimplemented!("serialize_unit_struct")
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok> {
        unimplemented!("serialize_unit_variant")
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!("serialize_newtype_struct")
    }

    fn serialize_newtype_variant<T: ?Sized>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        unimplemented!("serialize_newtype_variant")
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        unimplemented!("serialize_seq")
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!("serialize_tuple")
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
        unimplemented!("serialize_tuple_struct")
    }

    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant> {
        unimplemented!("serialize_tuple_variant")
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!("serialize_map")
    }

    fn serialize_struct_variant(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStructVariant> {
        unimplemented!("serialize_struct_variant")
    }   
}

impl<'a> serde::ser::SerializeSeq for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
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
    type Error = error::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
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
    type Error = error::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
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
    type Error = error::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
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
    type Error = error::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_key")
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
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
    type Error = error::Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_field")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut DocBufSerializer {
    type Ok = ();
    type Error = error::Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unimplemented!("serialize_field")
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!("end")
    }
}