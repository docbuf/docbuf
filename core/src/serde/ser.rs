use serde::{Serialize, Serializer};

use crate::vtable::*;
use crate::{error::Error, traits::DocBuf, Result};

#[derive(Debug)]
pub struct DocBufSerializer<'a> {
    pub vtable: &'static VTable<'static>,
    pub output: &'a mut Vec<u8>,
    pub current_item_index: u8,
    pub current_field_index: u8,
    pub previous_item_index: u8,
    pub current_item: Option<&'static VTableItem<'static>>,
    pub current_field: Option<&'static VTableField<'static>>,
    pub previous_item: Option<&'static VTableItem<'static>>,
}

impl<'a> DocBufSerializer<'a> {
    pub fn new(vtable: &'static VTable, output: &'a mut Vec<u8>) -> Self {
        // Clear the output buffer before serializing
        output.clear();

        Self {
            vtable,
            output,
            current_item_index: 0,
            current_field_index: 0,
            previous_item_index: 0,
            current_item: None,
            current_field: None,
            previous_item: None,
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

    pub fn encode_array_start(&mut self, num_elements: usize) -> Result<()> {
        self.current_field()?
            .encode_array_start(num_elements, &mut self.output)?;

        Ok(())
    }

    // Encode the beginning of a map structure
    pub fn encode_map_start(&mut self, num_entries: usize) -> Result<()> {
        self.current_field()?
            .encode_map_start(num_entries, &mut self.output)?;

        Ok(())
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

pub fn to_docbuf<T>(value: &T, buffer: &mut Vec<u8>) -> Result<()>
where
    T: Serialize + DocBuf + std::fmt::Debug,
{
    let mut serializer = DocBufSerializer::new(T::vtable()?, buffer);

    value.serialize(&mut serializer)?;

    // println!("Buffer: {:?}", buffer);

    Ok(())
}

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
                        self.previous_item_index = self.current_item_index;
                        self.current_item_index = vtable_struct.item_index;
                        self.previous_item = self.current_item;
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
        // println!("serialize_str: {}", v);

        self.encode_str(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        let field = self.current_field()?;

        // Shortcut validation for byte arrays
        if let FieldType::Bytes = field.field_type {
            self.output.push(v);

            Ok(())
        } else {
            // Encode the u8 as a numeric value
            self.encode_numeric(NumericValue::U8(v))
        }
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

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.encode_array_start(len.unwrap_or_default())?;

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        let field = self.current_field()?;

        if let FieldType::Bytes = field.field_type {
            // Encode the length of the byte array
            self.encode_array_start(len)?;
        }

        Ok(self)
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

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        // Encode the number of entries in the map
        self.encode_map_start(len.unwrap_or_default())?;

        Ok(self)
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

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, 'b> serde::ser::SerializeTuple for &'a mut DocBufSerializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
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

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        Ok(())
        // unimplemented!("serialize_key")
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Ok(())
        // unimplemented!("serialize_value")
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        key.serialize(&mut **self)?;

        value.serialize(&mut **self)?;

        // match &field.field_type {
        //     FieldType::HashMap { key, value } => key.serialize(self),
        //     _ => {
        //         return Err(Error::Serde(format!(
        //             "Expected a map, found a {:?}",
        //             field.field_type
        //         )));
        //     }
        // }

        // println!("Field: {:?}", field);

        // unimplemented!("serialize_entry")

        // let key = key.serialize(KeySerializer)?;
        // let value = value.serialize(ValueSerializer)?;

        // self.current_field = Some(Field {
        //     field_index: self.current_field_index,
        //     key,
        //     value,
        // });

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, 'b> serde::ser::SerializeStruct for &'a mut DocBufSerializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        // println!("Serialize field: {}", name);
        // println!("Current Item: {:?}", self.current_item);

        let field = match self.current_item {
            Some(VTableItem::Struct(field_struct)) => match field_struct.field_by_name(name) {
                Ok(field) => field,
                Err(_) => match self.previous_item {
                    Some(VTableItem::Struct(field_struct)) => {
                        match field_struct.field_by_name(name) {
                            Ok(field) => {
                                self.current_item = self.previous_item;
                                self.current_item_index = self.previous_item_index;
                                self.previous_item = None;
                                field
                            }
                            Err(_) => {
                                return Err(Error::Serde(format!("Field not found: {}", name)));
                            }
                        }
                    }
                    _ => {
                        return Err(Error::Serde(format!("Field not found: {}", name)));
                    }
                },
            },
            _ => match self
                .vtable
                .struct_by_index(self.current_item_index)?
                .field_by_name(name)
            {
                Ok(field) => field,
                Err(_) => {
                    // println!("Searching previous item");

                    // Try to search previous item
                    match self
                        .vtable
                        .struct_by_index(self.previous_item_index)?
                        .field_by_name(name)
                    {
                        Ok(field) => {
                            // Reset the current item to the parent struct
                            self.current_item = self.previous_item;
                            self.current_item_index = self.previous_item_index;
                            self.previous_item = None;

                            // Return the field
                            field
                        }
                        Err(_) => {
                            return Err(Error::Serde(format!("Field not found: {}", name)));
                        }
                    }
                }
            },
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
