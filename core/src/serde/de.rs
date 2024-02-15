use std::collections::HashMap;

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::{Deserialize, Deserializer};

use crate::vtable::*;
use crate::{error::Error, traits::DocBuf, Result};

#[derive(Debug, Clone)]
pub struct DocBufRawValues(HashMap<StructIndex, HashMap<FieldIndex, Vec<u8>>>);

impl DocBufRawValues {
    pub fn new() -> Self {
        DocBufRawValues(HashMap::new())
    }

    pub fn insert_value(
        &mut self,
        struct_index: StructIndex,
        field_index: FieldIndex,
        value: Vec<u8>,
    ) {
        self.0
            .entry(struct_index)
            .or_insert_with(HashMap::new)
            .insert(field_index, value);
    }

    pub fn get(&self, struct_index: StructIndex, field_index: FieldIndex) -> Option<&Vec<u8>> {
        self.0.get(&struct_index)?.get(&field_index)
    }

    // Check if the raw values is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // Remove the value from the raw values
    pub fn remove(
        &mut self,
        struct_index: StructIndex,
        field_index: FieldIndex,
    ) -> Option<Vec<u8>> {
        let structs = self.0.get_mut(&struct_index)?;
        let value = structs.remove(&field_index);

        // Remove the struct if it's empty
        if structs.is_empty() {
            self.0.remove(&struct_index);
        }

        value
    }

    pub fn parse_bytes_input(vtable: &VTable, input: &[u8]) -> Result<Self> {
        let mut current_struct_index = 0;
        let mut current_field_index = 0;

        let mut data = Self::new();
        let mut input = input.to_vec();

        while !input.is_empty() {
            println!("Parsing Bytes Input: {:#?}", input.len());
            match vtable.get_struct_field_by_index(current_struct_index, current_field_index) {
                Ok(field) => {
                    println!("Field: {:?}", field);
                    println!("Current Struct Index: {:#?}", current_struct_index);
                    println!("Current Field Index: {:#?}", current_field_index);

                    match &field.field_type {
                        FieldType::String => {
                            let field_length = usize::from_le_bytes([
                                input[0], input[1], input[2], input[3], 0, 0, 0, 0,
                            ]);
                            let field_end = 4 + field_length;
                            let field_data = input[4..field_end].to_vec();

                            data.insert_value(
                                current_struct_index,
                                current_field_index,
                                field_data,
                            );

                            input = input[field_end..].to_vec();
                        }
                        FieldType::U8 => {
                            let field_data = input[0..1].to_vec();

                            data.insert_value(
                                current_struct_index,
                                current_field_index,
                                field_data,
                            );

                            input = input[1..].to_vec();
                        }
                        FieldType::Struct(struct_name) => {
                            // unimplemented!(
                            //     "parse_bytes_input Struct Field Type: {:#?}",
                            //     struct_name
                            // );
                        }
                        _ => {
                            unimplemented!("parse_bytes_input Field Type: {:#?}", field.field_type);
                        }
                    }

                    current_field_index += 1;
                }
                _ => {
                    current_struct_index += 1;
                    current_field_index = 0;
                }
            }
        }

        Ok(data)
    }
}

impl<'de> From<&'de [u8]> for DocBufRawValues {
    fn from(input: &'de [u8]) -> Self {
        let mut data = input.to_vec();
        let mut raw_values = HashMap::new();

        while !data.is_empty() {
            let struct_index = data[0];
            let field_index = data[1];
            let field_length =
                usize::from_le_bytes([data[2], data[3], data[4], data[5], 0, 0, 0, 0]);
            let field_end = 6 + field_length;
            let field_data = data[6..field_end].to_vec();

            raw_values
                .entry(struct_index)
                .or_insert_with(HashMap::new)
                .insert(field_index, field_data);

            data = data[field_end..].to_vec();
        }

        DocBufRawValues(raw_values)
    }
}

#[derive(Debug)]
pub struct DocBufDeserializer {
    vtable: VTable,
    // input: &'de [u8],
    raw_values: DocBufRawValues,
    current_struct_index: StructIndex,
    current_field_index: FieldIndex,
}

impl<'de> DocBufDeserializer {
    pub fn from_docbuf(vtable: VTable, input: &'de [u8]) -> Result<Self> {
        let raw_values = DocBufRawValues::parse_bytes_input(&vtable, input)?;

        // println!("Raw Values: {:#?}", raw_values);

        Ok(DocBufDeserializer {
            vtable,
            // input,
            raw_values,
            current_struct_index: 0,
            current_field_index: 0,
        })
    }

    pub fn increment_current_field_index(&mut self) {
        self.current_field_index += 1;
    }
}

pub fn from_docbuf<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de> + DocBuf,
{
    let vtable = T::vtable()?;
    let mut deserializer = DocBufDeserializer::from_docbuf(vtable, input)?;

    // println!("Deserializer: {:#?}", deserializer);

    let t = T::deserialize(&mut deserializer)?;

    if deserializer.raw_values.is_empty() {
        Ok(t)
    } else {
        // println!("Raw Values: {:#?}", deserializer.raw_values);

        Err(Error::Serde("Unhandled trailing bytes".to_string()))
    }
}

impl<'de> serde::de::MapAccess<'de> for &mut DocBufDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // println!("next_key_seed");

        let field_data = self
            .raw_values
            .get(self.current_struct_index, self.current_field_index);

        if field_data.is_none() {
            // Field cannot be found for current struct and field indexes
            return Ok(None);
        }

        // let struct_info = self.vtable.struct_by_index(&self.current_struct_index)?;
        // let field = struct_info.field_by_index(&self.current_field_index)?;

        // let value = std::str::from_utf8(&field.field_name_as_bytes)?;

        // println!("map_access::next_key_seed Field Name: {:#?}", value);

        // let mut input = self.input.to_vec();
        // let mut iter = input.iter_mut();

        // let struct_index = iter
        //     .next()
        //     .ok_or(Error::Serde("No struct index".to_string()))?;
        // let vtable_struct = self.vtable.struct_by_index(struct_index)?;

        // // parse the field name in bytes as ut8
        // let field_index = iter
        //     .next()
        //     .ok_or(Error::Serde("No field index".to_string()))?;

        // let field = vtable_struct.field_by_index(field_index)?;

        // let value = std::str::from_utf8(&field.field_name_as_bytes)?;

        // println!("map_access::next_key_seed Field Name: {:#?}", value);

        // // remove the struct and field index from the input
        // self.input = &self.input[2..];

        seed.deserialize(&mut **self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // println!("next_value_seed");

        let data = self
            .raw_values
            .remove(self.current_struct_index, self.current_field_index)
            .unwrap_or_default();

        // println!("Reading Field Data: {:#?}", data);

        // if field.is_none() {
        //     // Field cannot be found for current struct and field indexes
        //     return Ok(None);
        // }

        seed.deserialize(&mut **self)
    }
}

impl<'de> SeqAccess<'de> for DocBufDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self).map(Some)
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut DocBufDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_any")
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_bool")
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_i8")
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_i16")
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_i32")
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("\n\nDeserialize u8");
        // println!("Current Struct Index: {:#?}", self.current_struct_index);
        // println!("Current Field Index: {:#?}", self.current_field_index);

        let field_data = self
            .raw_values
            .remove(self.current_struct_index, self.current_field_index)
            .unwrap_or_default();

        // println!("Field Data: {:#?}", field_data);

        let field_info = self
            .vtable
            .struct_by_index(&self.current_struct_index)?
            .field_by_index(&self.current_field_index)?;

        let name = field_info.field_name_as_string()?;

        // println!("Field Name: {:#?}", name);

        let value = field_data.first().unwrap_or(&0);
        // .ok_or(Error::Serde("No u8 value".to_string()))?;

        // Increment the field index
        self.increment_current_field_index();

        visitor.visit_u8(*value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_u16")
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_u32")
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_u64")
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_f32")
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_f64")
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_char")
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("\n\nDeserialize String");
        // println!("Current Struct Index: {:#?}", self.current_struct_index);
        // println!("Current Field Index: {:#?}", self.current_field_index);

        let field_data = self
            .raw_values
            .remove(self.current_struct_index, self.current_field_index)
            .unwrap_or_default();

        let field_info = self
            .vtable
            .struct_by_index(&self.current_struct_index)?
            .field_by_index(&self.current_field_index)?;

        let value = std::str::from_utf8(&field_data)?;

        // Increment the field index
        self.increment_current_field_index();

        visitor.visit_str(value)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("\n\nDeserialize String");
        // println!("Current Struct Index: {:#?}", self.current_struct_index);
        // println!("Current Field Index: {:#?}", self.current_field_index);

        let field_data = self
            .raw_values
            .remove(self.current_struct_index, self.current_field_index)
            .unwrap_or_default();

        let field_info = self
            .vtable
            .struct_by_index(&self.current_struct_index)?
            .field_by_index(&self.current_field_index)?;

        let value = std::str::from_utf8(&field_data)?;

        // Increment the field index
        self.increment_current_field_index();

        visitor.visit_string(value.to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_bytes")
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_byte_buf")
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_option")
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_unit")
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_unit_struct")
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_newtype_struct")
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("deserialize_seq: {:#?}", self.current_struct_index);

        let value = visitor.visit_seq(self)?;

        // println!("deserialize_seq: {:#?}", value);

        Ok(value)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_tuple")
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_tuple_struct")
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("deserialize_struct: {:#?}", name);

        // visit the struct name
        // let value = visitor.visit_newtype_struct(self)?;

        // println!("deserialize_struct: {:#?}", fields);

        // find the struct index for the name
        let current_struct = self.vtable.struct_by_name(name)?;

        // Set the current struct index;
        self.current_struct_index = current_struct.struct_index;
        // Reset the current field index
        self.current_field_index = 0;

        // println!("Current Struct Index: {:#?}", self.current_struct_index);

        // // find the field indexes for the struct
        // for field in fields {
        //     let VTableField {
        //         struct_index,
        //         field_index,
        //         field_type,
        //         ..
        //     } = current_struct.field_by_name(field)?;
        //     match field_type {
        //         FieldType::String => {
        //                 println!("deserialize_struct: Field Type: String");
        //             if let Some(value) = self.raw_values.remove(*struct_index, *field_index) {
        //                 // parse string from bytes
        //                 let value = std::str::from_utf8(&value)?;
        //                 println!("deserialize_struct: Field Value: {:#?}", value);
        //                 // visitor.visit_string(value.to_string())
        //                 // visitor.visit_string::<Error>(value.to_string())?;
        //             }
        //         }
        //         FieldType::Struct(name) => {
        //                 println!("deserialize_struct: Field Type: Struct");

        //             let name = std::str::from_utf8(name)?;

        //                 println!("deserialize_struct: Field Struct Type: {:#?}", name);

        //             if let Some(value) = self.raw_values.remove(*struct_index, *field_index) {
        //                 // parse string from bytes
        //                 let value = std::str::from_utf8(&value)?;
        //                 println!("deserialize_struct: Struct: {:#?}", value);
        //                 // visitor.visit_string(value.to_string())
        //             }
        //         }
        //         _ => {
        //             unimplemented!("deserialize_struct: Field Type: {:#?}", field_type)
        //         }
        //     }
        // }

        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_enum")
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)

        // let mut input = self.input.to_vec();
        // let mut iter = input.iter_mut();

        // let struct_index = iter
        //     .next()
        //     .ok_or(Error::Serde("No struct index".to_string()))?;
        // let vtable_struct = self.vtable.struct_by_index(struct_index)?;

        // // parse the field name in bytes as ut8
        // let field_index = iter
        //     .next()
        //     .ok_or(Error::Serde("No field index".to_string()))?;

        // let field = vtable_struct.field_by_index(field_index)?;

        // let value = std::str::from_utf8(&field.field_name_as_bytes)?;

        // println!("Input Length: {:#?}", self.input.len());
        // println!("Ending Input Length: {:#?}", input.len());

        // // remove the struct and field index from the input
        // self.input = &self.input[2..];

        // // visitor.visit_map()

        // visitor.visit_string(value.to_string())

        // unimplemented!("deserialize_identifier")
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_ignored_any")
    }
}
