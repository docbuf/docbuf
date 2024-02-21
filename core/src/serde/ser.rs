use serde::Serialize;

use crate::vtable::*;
use crate::{error::Error, traits::DocBuf, Result};

// pub enum Output {
//     Static(&'static mut Vec<u8>),
//     Dynamic(Vec<u8>),
// }

// impl Output {
//     pub fn as_mut(&mut self) -> &mut Vec<u8> {
//         match self {
//             Output::Static(buf) => *buf,
//             Output::Dynamic(buf) => buf,
//         }
//     }

//     pub fn to_vec(self) -> Vec<u8> {
//         match self {
//             Output::Static(buf) => buf.to_vec(),
//             Output::Dynamic(buf) => buf,
//         }
//     }
// }

pub struct DocBufSerializer<'a> {
    pub vtable: &'static VTable<'static>,
    pub output: &'a mut Vec<u8>,
    pub current_item_index: u8,
    pub current_field_index: u8,
    pub current_item: Option<&'static VTableItem<'static>>,
    pub current_field: Option<&'static VTableField<'static>>,
}

impl<'a> DocBufSerializer<'a> {
    pub fn new(vtable: &'static VTable, output: &'a mut Vec<u8>) -> Self {
        // Clear the output buffer
        output.clear();

        Self {
            vtable,
            output,
            current_item_index: 0,
            current_field_index: 0,
            current_item: None,
            current_field: None,
        }
    }

    // Return the current field or find it in the vtable based on the
    // current_item_index and current_field_index
    pub fn current_field(&self) -> Result<&'static VTableField<'static>> {
        Ok(match self.current_field {
            Some(field) => field,
            _ => self
                .vtable
                .struct_by_index(self.current_item_index)?
                .field_by_index(&self.current_field_index)?,
        })
    }

    // Encode the &str data into the output vector
    pub fn encode_str(&mut self, data: &str) -> Result<()> {
        self.current_field()?.encode_str(data, &mut self.output)?;

        Ok(())
    }

    // Encode the &str data into the output vector
    pub fn encode_numeric(&mut self, data: NumericValue) -> Result<()> {
        self.current_field()?
            .encode_numeric_value(data, &mut self.output)?;

        Ok(())
    }

    // // Encode the data into the output vector
    // pub fn encode(&mut self, data: impl AsRef<[u8]>) -> Result<()> {
    //     let field = match self.current_field {
    //         Some(field) => field,
    //         _ => self
    //             .vtable
    //             .struct_by_index(self.current_item_index)?
    //             .field_by_index(&self.current_field_index)?,
    //     };

    //     field.encode(data.as_ref(), &mut self.output)?;

    //     Ok(())
    // }
}

pub fn to_docbuf_writer<T>(value: &T, buffer: &mut Vec<u8>) -> Result<()>
where
    T: Serialize + DocBuf + std::fmt::Debug,
{
    let mut serializer = DocBufSerializer::new(T::vtable()?, buffer);

    value.serialize(&mut serializer)?;

    Ok(())
}

// pub fn to_docbuf<T>(value: &T) -> Result<Vec<u8>>
// where
//     T: Serialize + DocBuf + std::fmt::Debug,
// {
//     let mut serializer =
//         DocBufSerializer::new(T::vtable()?, Output::Dynamic(Vec::with_capacity(1024)));

//     value.serialize(&mut serializer)?;

//     Ok(serializer.output.to_vec())
// }

impl<'a, 'b> serde::ser::Serializer for &'a mut DocBufSerializer<'b> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self> {
        for vtable_item in self.vtable.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.struct_name == name {
                        self.current_item_index = vtable_struct.item_index;
                        self.current_item = Some(vtable_item);
                        return Ok(self);
                    }
                } // _ => {}
            }
        }

        Err(Error::Serde(format!(
            "Struct {} not found in the vtable",
            name
        )))
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

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::F64(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::I8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::I64(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::I128(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.encode_str(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::U16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::U64(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        self.encode_numeric(NumericValue::U128(v))
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

impl<'a, 'b> serde::ser::SerializeSeq for &'a mut DocBufSerializer<'b> {
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

impl<'a, 'b> serde::ser::SerializeTuple for &'a mut DocBufSerializer<'b> {
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

impl<'a, 'b> serde::ser::SerializeTupleStruct for &'a mut DocBufSerializer<'b> {
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

impl<'a, 'b> serde::ser::SerializeTupleVariant for &'a mut DocBufSerializer<'b> {
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

impl<'a, 'b> serde::ser::SerializeMap for &'a mut DocBufSerializer<'b> {
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

impl<'a, 'b> serde::ser::SerializeStruct for &'a mut DocBufSerializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let field = match self.current_item {
            Some(VTableItem::Struct(field_struct)) => field_struct.field_by_name(name)?,
            _ => self
                .vtable
                .struct_by_index(self.current_item_index)?
                .field_by_name(name)?,
        };

        self.current_field_index = field.field_index;
        self.current_field = Some(field);

        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Update the current index
        Ok(())
    }
}

impl<'a, 'b> serde::ser::SerializeStructVariant for &'a mut DocBufSerializer<'b> {
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
