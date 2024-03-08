use std::collections::HashSet;

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
// use tracing::{debug, instrument};

use crate::vtable::*;
use crate::{
    error::Error,
    traits::{DocBuf, DocBufDecodeField},
    Result,
};

#[derive(Debug)]
pub struct DocBufDeserializer<'a> {
    vtable: &'static VTable<'static>,
    buffer: &'a mut Vec<u8>,
    // raw_values: DocBufRawValues,
    current_item_index: VTableItemIndex,
    current_field_index: VTableFieldIndex,
    current_item: Option<&'static VTableItem<'static>>,
    current_field: Option<&'static VTableField<'static>>,
    // has_visited: HashSet<(VTableItemIndex, VTableFieldIndex)>,
    has_descended: bool,
    // Used for Hash Map deserialization.
    remaining_map_entries: Option<u32>,
}

impl<'de> DocBufDeserializer<'de> {
    pub fn from_docbuf(vtable: &'static VTable, buffer: &'de mut Vec<u8>) -> Result<Self> {
        Ok(DocBufDeserializer {
            vtable,
            buffer,
            current_item_index: 0,
            current_field_index: 0,
            current_item: None,
            current_field: None,
            has_descended: false,
            // has_visited: HashSet::with_capacity(vtable.num_items as usize * 10),
            remaining_map_entries: None,
        })
    }

    pub fn next_field(&mut self) -> Result<()> {
        if let Some(remaining) = self.remaining_map_entries {
            if remaining > 0 {
                return Ok(());
            }
        }

        // Skip if the field has already been visited.
        // if self
        //     .has_visited
        //     .contains(&(self.current_item_index, self.current_field_index))
        // {
        //     self.current_field_index += 1;
        //     self.current_field = None;
        //     return Ok(());
        // }

        // Mark the current item as visited.
        // self.has_visited
        //     .insert((self.current_item_index, self.current_field_index));

        let item = self.current_item()?;

        // println!("Item: {:?}", item);

        let num_items = self.vtable.num_items;

        // println!("Num Items: {}", num_items);

        match item {
            VTableItem::Struct(s) => {
                // println!("Num Fields: {:?}", s.num_fields);

                if s.num_fields <= self.current_field_index
                    && self.current_item_index == num_items - 1
                    && self.has_descended
                {
                    // println!("Return Early");
                    return Ok(());
                }

                // println!("Search field by index");
                let field = s.field_by_index(&self.current_field_index)?;

                // println!("Field: {:?}", field);

                if let VTableFieldType::Struct(_) = field.field_type {
                    // println!("Check for descent");
                    if !self.has_descended {
                        // println!("Descending, Decrementing item index");
                        self.current_item_index -= 1;
                        self.current_field_index = 0;
                        self.current_item = None;
                        self.current_field = None;
                        return Ok(());
                    }
                }

                if self.current_field_index < s.num_fields - 1 {
                    // println!("Incrementing Field Index");
                    self.current_field_index += 1;
                    self.current_field = None;
                } else if self.current_item_index > 0 && !self.has_descended {
                    // println!("Decrementing Item Index");
                    self.current_item_index -= 1;
                    self.current_field_index = 0;
                    self.current_field = None;
                    self.current_item = None;
                } else if self.current_item_index == 0
                    && self.current_field_index == s.num_fields - 1
                    && !self.has_descended
                {
                    // println!("Has Descended, Incrementing Item Index");
                    self.has_descended = true;
                    self.current_item_index += 1;
                    self.current_field_index = 0;
                    self.current_item = None;
                    self.current_field = None;
                    return self.next_field();
                    // } else if self.current_item_index < num_items - 1 && self.has_descended {
                    //     // println!("Incrementing Item Index");
                    //     self.current_item_index += 1;
                    //     self.current_field_index = 0;
                    //     self.current_item = None;
                    //     self.current_field = None;
                }
                // else {
                //     // println!("No More Fields");
                //     // return Err(Error::Serde("No more fields".to_string()));
                //     return Ok(());
                // }
            }
        }

        Ok(())
    }

    // Return the current field or find it in the vtable based on the
    // current_item_index and current_field_index
    pub fn current_field(&mut self) -> Result<&'static VTableField<'static>> {
        // println!("Current Field");
        Ok(match self.current_field {
            Some(field) => field,
            _ => self.set_current_field()?,
        })
    }

    pub fn set_current_field(&mut self) -> Result<&'static VTableField<'static>> {
        // println!("Set Current Field");
        let field = self
            .vtable
            .struct_by_index(self.current_item_index)?
            .field_by_index(&self.current_field_index)?;

        self.current_field = Some(field);
        Ok(field)
    }

    // Return the current item or find it in the vtable based on the current item index
    pub fn current_item(&mut self) -> Result<&'static VTableItem<'static>> {
        Ok(match self.current_item {
            Some(item) => item,
            _ => self.set_current_item()?,
        })
    }

    // Set the current item based on the current item index
    pub fn set_current_item(&mut self) -> Result<&'static VTableItem<'static>> {
        let item = self.vtable.item_by_index(self.current_item_index)?;
        self.current_item = Some(item);
        Ok(item)
    }
}

pub fn from_docbuf<'de, T>(buffer: &'de mut Vec<u8>) -> Result<T>
where
    T: Deserialize<'de> + DocBuf,
{
    let vtable = T::vtable()?;
    let mut deserializer = DocBufDeserializer::from_docbuf(vtable, buffer)?;

    // println!("Deserializer: {:#?}", deserializer);

    let t = T::deserialize(&mut deserializer)?;

    if deserializer.buffer.is_empty() {
        Ok(t)
    } else {
        // println!("Raw Values: {:#?}", deserializer.raw_values);

        Err(Error::Serde("Unhandled trailing bytes".to_string()))
    }
}

impl<'de> MapAccess<'de> for &mut DocBufDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // println!(
        //     "Deserializing Map Key: {} {}",
        //     self.current_item_index, self.current_field_index
        // );

        if self.buffer.is_empty()
        // || self
        //     .raw_values
        //     .get(self.current_item_index, self.current_field_index)
        //     .is_none()
        {
            // self.next_field()?;
            return Ok(None);
        }

        // println!("Remaining Bytes: {:?}", self.buffer);

        let field = self.current_field()?;

        // Set the hash map field items to the length of the data.
        if let VTableFieldType::HashMap { .. } = &field.field_type {
            // println!("Processing Hash Map Key");
            // println!("Remaining Field Items: {:?}", self.remaining_map_entries);
            match self.remaining_map_entries {
                None => {
                    self.remaining_map_entries = Some(u32::from_le_bytes([
                        self.buffer[0],
                        self.buffer[1],
                        self.buffer[2],
                        0,
                    ]));

                    // drain the length from the buffer
                    self.buffer.drain(0..3);

                    // unimplemented!("Set Remaining Field Items: {:?}", self.buffer)
                }
                Some(0) => {
                    // println!("Finished Hash Map");

                    // println!("Remaining Bytes: {:?}", self.buffer);

                    self.remaining_map_entries = None;
                    self.next_field()?;
                    return Ok(None);
                }
                _ => {}
            }
        }

        seed.deserialize(&mut **self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut **self);

        // Decrement the remaining field items if it is set
        if let VTableFieldType::HashMap { .. } = &self.current_field()?.field_type {
            // println!("Processing Hash Map Value");
            // Decrement the remaining field items if it is set
            self.remaining_map_entries.as_mut().map(|x| *x -= 1);
        };

        value
    }
}

impl<'de> SeqAccess<'de> for DocBufDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // println!(
        //     "Next Element Seed: {} {}",
        //     self.current_item_index, self.current_field_index
        // );

        if self.buffer.is_empty() {
            // println!("Raw Values is empty");
            return Ok(None);
        }

        let field = self.current_field()?;

        // println!("Field: {:?}", field);

        match &field.field_type {
            VTableFieldType::Struct(_) => {
                // println!("Struct Field");
                self.next_field()?;
            }
            _ => {
                //

                // let field_data = field.decode(self.buffer)?;

                // println!("Field Data: {:?}", field_data);

                // if field_data.is_empty() {
                //     // println!("Field Data is None");
                //     // self.next_field()?;
                //     return Ok(None);
                // }
            }
        }

        seed.deserialize(&mut *self).map(Some)
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut DocBufDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!(
        //     "Deserializing Any: {} {}",
        //     self.current_item_index, self.current_field_index
        // );

        let field = self.current_field()?;

        // println!("Field: {:?}", field);
        // println!("Remaining Bytes: {:?}", self.buffer);

        match &field.field_type {
            VTableFieldType::Struct(s) => {
                self.next_field()?;
                self.deserialize_map(visitor)
            }
            // VTableFieldType::Array => self.deserialize_seq(visitor),
            // VTableFieldType::Map => self.deserialize_map(visitor),
            VTableFieldType::Bool => self.deserialize_bool(visitor),
            VTableFieldType::I8 => self.deserialize_i8(visitor),
            VTableFieldType::I16 => self.deserialize_i16(visitor),
            VTableFieldType::I32 => self.deserialize_i32(visitor),
            VTableFieldType::I64 | VTableFieldType::ISIZE => self.deserialize_i64(visitor),
            VTableFieldType::I128 => self.deserialize_i128(visitor),
            VTableFieldType::U8 => self.deserialize_u8(visitor),
            VTableFieldType::U16 => self.deserialize_u16(visitor),
            VTableFieldType::U32 => self.deserialize_u32(visitor),
            VTableFieldType::U64 | VTableFieldType::USIZE => self.deserialize_u64(visitor),
            VTableFieldType::U128 => self.deserialize_u128(visitor),
            VTableFieldType::F32 => self.deserialize_f32(visitor),
            VTableFieldType::F64 => self.deserialize_f64(visitor),
            VTableFieldType::String => self.deserialize_string(visitor),
            VTableFieldType::Bytes => self.deserialize_bytes(visitor),
            VTableFieldType::Str => self.deserialize_str(visitor),
            VTableFieldType::Vec => self.deserialize_seq(visitor),
            VTableFieldType::HashMap { key, value } => visitor.visit_map(self),
            // match self.remaining_map_entries {
            //     None => visitor.visit_map(self),
            //     Some(_) => match (key.as_ref(), value.as_ref()) {
            //         (VTableFieldType::String, VTableFieldType::String) => {
            //             self.deserialize_string(visitor)

            //                 self.next_key

            //         }
            //         _ => unimplemented!("HashMap Key/Value: {:?}/{:?}", key, value),
            //     },
            // },
            // VTableFieldType::Option => self.deserialize_option(visitor),
            // VTableFieldType::Unit => self.deserialize_unit(visitor),
            // VTableFieldType::UnitStruct => self.deserialize_unit_struct(field.name, visitor),
            // VTableFieldType::NewtypeStruct => self.deserialize_newtype_struct(field.name, visitor),
            // VTableFieldType::Tuple => self.deserialize_tuple(field.fields.len(), visitor),
            // VTableFieldType::TupleStruct => {
            //     self.deserialize_tuple_struct(field.name, field.fields.len(), visitor)
            // }
            // VTableFieldType::Enum => self.deserialize_enum(field.name, &field.fields, visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_bool(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // let value = i16::from_le_bytes([field_data[0], field_data[1]]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_i128(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // println!("Raw Values: {:?}", self.raw_values);
        // println!(
        //     "Current Item Index: {}, Current Field Index: {}",
        //     self.current_item_index, self.current_field_index
        // );
        //

        // println!(
        //     "Deserializing u8: {} {}",
        //     self.current_item_index, self.current_field_index
        // );

        // println!(
        //     "Deserializing U8: {} {}",
        //     self.current_item_index, self.current_field_index
        // );

        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_u64(value)
    }

    /// Hint that the `Deserialize` type is expecting an `u128` value.
    ///
    /// The default behavior unconditionally returns an error.
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_u128(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        // Increment the field index
        self.next_field()?;

        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_char")
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field = self.current_field()?;

        let value: String = field.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_str(value.as_str())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field = self.current_field()?;

        let value = field.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_string(value)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value: Vec<u8> = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_bytes(value.as_slice())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_byte_buf(value)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_option")
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_unit")
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_unit_struct")
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_newtype_struct")
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(self)?;

        Ok(value)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
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
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let current_struct = self.vtable.struct_by_name(name)?;

        // Set the current struct index;
        self.current_item_index = current_struct.item_index;
        // Reset the current field index
        self.current_field_index = 0;

        self.deserialize_seq(visitor)
        // self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
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
        // println!("Deserialize Identifier");

        let field = self.current_field()?;

        // println!("Deserializing Field: {:?}", field.field_name);

        match &field.field_type {
            VTableFieldType::Struct(_) => {
                self.next_field()?;
                visitor.visit_str(field.field_name)
            }
            VTableFieldType::HashMap { key, .. } => match key.as_ref() {
                VTableFieldType::String => {
                    // println!("Deserialize Identifier String for hash map");
                    let data = field.decode(self.buffer)?;

                    visitor.visit_string(data)
                }
                _ => {
                    unimplemented!("deserialize_identifier for hash map key type: {:?}", key);
                }
            },
            _ => visitor.visit_str(field.field_name),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
