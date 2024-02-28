use std::collections::HashSet;

use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::Deserialize;

use crate::vtable::*;
use crate::{error::Error, traits::DocBuf, Result};

#[derive(Debug)]
pub struct DocBufDeserializer {
    vtable: &'static VTable<'static>,
    raw_values: DocBufRawValues,
    current_item_index: VTableItemIndex,
    current_field_index: FieldIndex,
    current_item: Option<&'static VTableItem<'static>>,
    current_field: Option<&'static VTableField<'static>>,
    has_visited: HashSet<(VTableItemIndex, FieldIndex)>,
    has_descended: bool,
}

impl<'de> DocBufDeserializer {
    pub fn from_docbuf(vtable: &'static VTable, input: &'de [u8]) -> Result<Self> {
        // Parse the raw values according to the vtable rules
        let raw_values = vtable.parse_raw_values(input)?;

        Ok(DocBufDeserializer {
            vtable,
            raw_values,
            current_item_index: 0,
            current_field_index: 0,
            current_item: None,
            current_field: None,
            has_descended: false,
            has_visited: HashSet::new(),
        })
    }

    pub fn next_field(&mut self) -> Result<()> {
        // Skip if the field has already been visited.
        if self
            .has_visited
            .contains(&(self.current_item_index, self.current_field_index))
        {
            self.current_field_index += 1;
            self.current_field = None;
            return self.next_field();
        }

        // Mark the current item as visited.
        self.has_visited
            .insert((self.current_item_index, self.current_field_index));

        let item = self.current_item()?;

        let num_items = self.vtable.num_items;

        match item {
            VTableItem::Struct(s) => {
                let field = s.field_by_index(&self.current_field_index)?;

                if let FieldType::Struct(_) = field.field_type {
                    if !self.has_descended {
                        self.current_item_index -= 1;
                        self.current_field_index = 0;
                        self.current_item = None;
                        self.current_field = None;
                        return Ok(());
                    }
                }

                if self.current_field_index < s.num_fields - 1 {
                    println!("Incrementing Field Index");
                    self.current_field_index += 1;
                    self.current_field = None;
                } else if self.current_item_index > 0 && !self.has_descended {
                    println!("Decrementing Item Index");
                    self.current_item_index -= 1;
                    self.current_field_index = 0;
                    self.current_field = None;
                    self.current_item = None;
                } else if self.current_item_index == 0
                    && self.current_field_index == s.num_fields - 1
                    && !self.has_descended
                {
                    println!("Has Descended");
                    self.has_descended = true;
                    self.current_item_index += 1;
                    self.current_field_index = 0;
                    self.current_item = None;
                    self.current_field = None;
                } else if self.current_item_index < num_items - 1 && self.has_descended {
                    println!("Incrementing Item Index");
                    self.current_item_index += 1;
                    self.current_field_index = 0;
                    self.current_item = None;
                    self.current_field = None;
                } else {
                    // println!("No More Fields");
                    // return Err(Error::Serde("No more fields".to_string()));
                    return Ok(());
                }
            }
        }

        // Check if the next item and field have been visited, skip if so
        if self
            .has_visited
            .contains(&(self.current_item_index, self.current_field_index))
        {
            println!("Has visited");
            self.next_field()?;
        }

        Ok(())
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

    // Return the current item or find it in the vtable based on the current item index
    pub fn current_item(&self) -> Result<&'static VTableItem<'static>> {
        Ok(match self.current_item {
            Some(item) => item,
            _ => self.vtable.item_by_index(self.current_item_index)?,
        })
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
        // println!("Deserializing Map Key");

        let field_data = self
            .raw_values
            .get(self.current_item_index, self.current_field_index);

        // println!("Field Data: {:?}", field_data);

        if field_data.is_none() {
            self.next_field()?;

            // Field cannot be found for current struct and field indexes
            return Ok(None);
        }

        seed.deserialize(&mut **self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // println!("Deserializing Map Value");

        // let mut data = self
        //     .raw_values
        //     .remove(self.current_item_index, self.current_field_index)
        //     .unwrap_or_default();

        // println!("Data: {:?}", data);

        // // Drain the key and value from the data
        // let key_length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        // data.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES + key_length as usize);

        // if !data.is_empty() {
        //     self.raw_values
        //         .insert(self.current_item_index, self.current_field_index, data);
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
        println!("Next element Seed");

        if self.raw_values.is_empty() {
            return Ok(None);
        }

        seed.deserialize(&mut *self).map(Some)
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut DocBufDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_any")
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_bool")
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = i8::from_le_bytes([field_data[0]]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = i16::from_le_bytes([field_data[0], field_data[1]]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value =
            i32::from_le_bytes([field_data[0], field_data[1], field_data[2], field_data[3]]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = i64::from_le_bytes([
            field_data[0],
            field_data[1],
            field_data[2],
            field_data[3],
            field_data[4],
            field_data[5],
            field_data[6],
            field_data[7],
        ]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = i128::from_le_bytes([
            field_data[0],
            field_data[1],
            field_data[2],
            field_data[3],
            field_data[4],
            field_data[5],
            field_data[6],
            field_data[7],
            field_data[8],
            field_data[9],
            field_data[10],
            field_data[11],
            field_data[12],
            field_data[13],
            field_data[14],
            field_data[15],
        ]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_i128(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("Raw Values: {:?}", self.raw_values);
        println!(
            "Current Item Index: {}, Current Field Index: {}",
            self.current_item_index, self.current_field_index
        );

        let mut field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        //
        // let value = field_data[0]; //u8::from_le_bytes([]);

        // Drain the first byte from the field data
        let value = field_data
            .drain(0..1)
            .nth(0)
            .ok_or(Error::Serde("field data is empty.".to_string()))?;

        // If there is remaining field data, re-insert it
        if field_data.len() > 0 {
            // re-insert the remaining field data
            self.raw_values.insert(
                self.current_item_index,
                self.current_field_index,
                field_data,
            );
        } else {
            // Increment the field index
            self.next_field()?;
        }

        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = u16::from_le_bytes([field_data[0], field_data[1]]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value =
            u32::from_le_bytes([field_data[0], field_data[1], field_data[2], field_data[3]]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = u64::from_le_bytes([
            field_data[0],
            field_data[1],
            field_data[2],
            field_data[3],
            field_data[4],
            field_data[5],
            field_data[6],
            field_data[7],
        ]);

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
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = u128::from_le_bytes([
            field_data[0],
            field_data[1],
            field_data[2],
            field_data[3],
            field_data[4],
            field_data[5],
            field_data[6],
            field_data[7],
            field_data[8],
            field_data[9],
            field_data[10],
            field_data[11],
            field_data[12],
            field_data[13],
            field_data[14],
            field_data[15],
        ]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_u128(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value =
            f32::from_le_bytes([field_data[0], field_data[1], field_data[2], field_data[3]]);

        // Increment the field index
        self.next_field()?;

        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let value = f64::from_le_bytes([
            field_data[0],
            field_data[1],
            field_data[2],
            field_data[3],
            field_data[4],
            field_data[5],
            field_data[6],
            field_data[7],
        ]);

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
        let mut data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        let field = self.current_field()?;

        let value = match &field.field_type {
            FieldType::String => {
                // Increment the field index
                self.next_field()?;

                println!("deserialize string: Data: {:?}", data);

                String::from_utf8(data)?
            }
            FieldType::HashMap { key, value } => {
                let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

                let value = data
                    [DEFAULT_FIELD_LENGTH_LE_BYTES..DEFAULT_FIELD_LENGTH_LE_BYTES + length]
                    .to_owned();

                println!("Value: {:?}", value);

                let value = String::from_utf8(value)?;

                data.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES + length);

                // Re-add the remainder data to the raw values
                if !data.is_empty() {
                    self.raw_values
                        .insert(self.current_item_index, self.current_field_index, data);
                }

                value
            }
            _ => {
                return Err(Error::Serde(format!(
                    "Expected field type to be String or HashMap, found {:?}",
                    field.field_type
                )))
            }
        };

        visitor.visit_str(&value)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // let field = self.current_field()?;

        let field = self
            .vtable
            .struct_by_index(self.current_item_index)?
            .field_by_index(&self.current_field_index)?;

        let mut field_data = self
            .raw_values
            .remove(self.current_item_index, self.current_field_index)
            .unwrap_or_default();

        println!("Field: {:?}", field);

        let value = match &field.field_type {
            FieldType::String => {
                // Increment the field index
                self.next_field()?;

                println!("Data: {:?}", field_data);

                String::from_utf8(field_data)?
            }
            FieldType::HashMap { key, value } => {
                let length = u32::from_le_bytes([
                    field_data[0],
                    field_data[1],
                    field_data[2],
                    field_data[3],
                ]) as usize;

                let value = &field_data
                    [DEFAULT_FIELD_LENGTH_LE_BYTES..DEFAULT_FIELD_LENGTH_LE_BYTES + length];

                println!("HashMap Data: {:?}", value);

                let value = String::from_utf8(value.to_owned())?;

                field_data.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES + length);

                if !field_data.is_empty() {
                    self.raw_values.insert(
                        self.current_item_index,
                        self.current_field_index,
                        field_data,
                    );
                }

                value
            }
            _ => {
                return Err(Error::Serde(format!(
                    "Expected field type to be String or HashMap, found {:?}",
                    field.field_type
                )))
            }
        };

        visitor.visit_string(value)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_bytes")
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_byte_buf")
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

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_tuple")
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
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("deserialize_ignored_any")
    }
}
