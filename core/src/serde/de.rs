use std::collections::HashSet;

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use tracing::debug;

use crate::vtable::*;
use crate::{
    error::Error,
    traits::{DocBuf, DocBufDecodeField},
    Result,
};

#[derive(Debug)]
pub struct DocBufDeserializer<'a> {
    vtable: &'static VTable,
    buffer: &'a mut Vec<u8>,
    current_item_index: VTableItemIndex,
    current_field_index: VTableFieldIndex,
    current_item: Option<&'static VTableItem>,
    current_field: Option<&'static VTableField>,
    // optional_field: Option<VTableFieldType>,
    has_visited: HashSet<(u8, u8)>,
    has_descended: bool,
    remaining_items: Option<u32>,
}

impl<'de> DocBufDeserializer<'de> {
    pub fn new(vtable: &'static VTable, buffer: &'de mut Vec<u8>) -> Result<Self> {
        Ok(DocBufDeserializer {
            vtable,
            buffer,
            current_item_index: 0,
            current_field_index: 0,
            current_item: None,
            current_field: None,
            // optional_field: None,
            has_visited: HashSet::with_capacity(vtable.num_fields as usize),
            has_descended: false,
            remaining_items: None,
        })
    }

    #[inline]
    pub fn next_field(&mut self) -> Result<()> {
        println!(
            "Next Field; Current Field Index: {}, Field: {:?}",
            self.current_field_index, self.current_field
        );

        if let Some(remaining) = self.remaining_items {
            if remaining > 0 {
                return Ok(());
            }
        }

        match self.current_field {
            None => {
                // If there is no current field, but at least one remaining item, set the remaining items to None.

                // if let Some(remaining) = self.remaining_items {
                //     if remaining == 1 {
                //         self.remaining_items = None;
                //     }
                // }

                // let mut not_visited = self
                //     .vtable
                //     .fields()
                //     .0
                //     .into_iter()
                //     .filter(|field| !self.has_visited.contains(&(field.item_index, field.index)))
                //     .map(|f| (f.item_index, f.index))
                //     // TODO: Remove allocation
                //     .collect::<Vec<_>>();

                // not_visited.sort();

                // println!("Not Visited: {:?}", not_visited);

                // if let Some((item_index, field_index)) = not_visited.first() {
                //     let field = self
                //         .vtable
                //         .get_item_field_by_index(*item_index, *field_index)?;

                //     println!("Field Not Visited: {:?}", field);

                //     if let VTableFieldType::Vec(_) = &field.r#type {
                //         self.has_visited.insert((field.item_index, field.index));
                //         self.set_remaining_items(&field.r#type)?;
                //     }

                //     self.current_item_index = *item_index;
                //     self.current_field_index = *field_index;
                //     self.current_item = None;
                //     self.current_field = None;

                //     println!(
                //         "Setting Current Field To: {}, {}",
                //         self.current_item_index, self.current_field_index
                //     );

                //     return Ok(());
                // } else {
                //     println!("Buffer: {:?}", self.buffer);
                //     panic!("End of fields")
                // }
            }
            Some(field) => {
                match &field.r#type {
                    VTableFieldType::HashMap { .. } => {
                        println!("Skipping remaining items for field {:?}", field);
                        if let Some(remaining) = self.remaining_items {
                            if remaining > 0 {
                                return Ok(());
                            }
                        }
                    }
                    _ => {}
                }

                // Mark the field as visited.
                self.has_visited.insert((field.item_index, field.index));

                // Set the current field index and item index if they do not match.
                if self.current_field_index != field.index
                    && self.current_item_index != field.item_index
                {
                    eprintln!("Current Field Index and Current Field Index do not match!");
                    self.current_field_index = field.index;
                    self.current_item_index = field.item_index;
                }
            }
        }

        // Reset the optional field if it is set.
        // self.optional_field = None;

        let item = self.current_item()?;

        println!("\n\nCurrent Item: {item:?}\n\n");

        let num_items = self.vtable.num_items;

        match item {
            VTableItem::Struct(s) => {
                if s.num_fields <= self.current_field_index
                    && self.current_item_index == num_items - 1
                    && self.has_descended
                {
                    // println!("Current Item Index: {}", self.current_item_index);

                    // panic!("Exiting Early");
                    return Ok(());
                }

                let field = s.field_by_index(&self.current_field_index)?;

                println!("Current Field: {:?}", field);
                println!("Buffer: {:?}", self.buffer);

                if !self.has_visited.contains(&(field.item_index, field.index)) {
                    if let VTableFieldType::Struct(_) = field.r#type {
                        println!("Decrementing item index.");
                        self.current_item_index -= 1;
                        self.current_field_index = 0;
                        self.current_item = None;
                        self.current_field = None;

                        return Ok(());
                    }

                    if let VTableFieldType::Option(opt) = &field.r#type {
                        if let VTableFieldType::Struct(_) = **opt {
                            println!("\n\nFound Optional Struct field: {:?}\n\n", field);
                            println!("Decrementing item index.");
                            self.current_item_index -= 1;
                            self.current_field_index = 0;
                            self.current_item = None;
                            self.current_field = None;
                            return Ok(());
                        }
                    }

                    if let VTableFieldType::Vec(_) = field.r#type {
                        println!("\n\nFound Vec field: {:?}\n\n", field);
                        println!("Current Item Index: {:?}", self.current_item_index);
                        println!("Remaining Items: {:?}", self.remaining_items);
                        if self.remaining_items.is_some() {
                            if self.current_item_index > 0 {
                                println!("Decrementing item index.");
                                self.current_item_index -= 1;
                                self.current_field_index = 0;
                                self.current_item = None;
                                self.current_field = None;
                            }

                            return Ok(());
                        }

                        // match self.remaining_items.as_ref() {
                        //     Some(remaining_items) => {
                        //        // Decrement the remaining items.
                        //        *remaining_items -= 1;

                        //        // Check if the remaining items have been exhausted.
                        //        if *remaining_items == 0 {
                        //            self.remaining_items = None;
                        //        }
                        //     }
                        //     _ => {
                        //         // println!("Ignoring Empty Vec");
                        //         // self.current_item_index = field.item_index;
                        //         // self.current_field_index = 0;
                        //         // return Ok(());
                        //     }
                        // }
                    }
                }

                // Increment the field if there are more fields.
                if self.current_field_index < s.num_fields - 1 {
                    println!("Incrementing field index.");
                    self.current_field_index += 1;
                    self.current_field = None;
                // Decrement the item index if the current item has no more fields,
                // and the current item index has not yet reached the zeroth index.
                } else if self.current_item_index > 0 && !self.has_descended {
                    println!("Decrementing item index.");
                    self.current_item_index -= 1;
                    self.current_field_index = 0;
                    self.current_field = None;
                    self.current_item = None;
                // Increment the item index if the current item has reached the zeroth index,
                } else if self.current_item_index == 0
                    && self.current_field_index == s.num_fields - 1
                    // && !self.has_descended
                    && self.current_item_index < num_items - 1
                {
                    println!("Incrementing item index.");
                    self.has_descended = true;
                    self.current_item_index += 1;
                    self.current_field_index = 0;
                    self.current_item = None;
                    self.current_field = None;
                    return self.next_field();
                } else {
                    println!("Current Item Index: {:?}", self.current_item_index);
                    println!("Current Field Index: {:?}", self.current_field_index);
                    println!("Current Item: {:?}", self.current_item);

                    println!("Buffer: {:?}", self.buffer);
                }
            }
        }

        Ok(())
    }

    // Return the current field or find it in the vtable based on the
    // current_item_index and current_field_index
    #[inline]
    pub fn current_field(&mut self) -> Result<&'static VTableField> {
        Ok(match self.current_field {
            Some(field) => field,
            _ => self.set_current_field()?,
        })
    }

    #[inline]
    pub fn set_remaining_items(&mut self, field_type: &VTableFieldType) -> Result<()> {
        println!(
            "set_remaining_items {:?}; Buffer: {:?}; Field Type: {:?}",
            self.remaining_items, self.buffer, field_type
        );

        if self.buffer.is_empty() {
            self.remaining_items = None;
            return Ok(());
        }

        match (&field_type, self.remaining_items) {
            (VTableFieldType::Uuid, None) => {
                self.remaining_items = Some(15);
            }
            (VTableFieldType::Bytes, None) => {
                let data_len = u32::from_le_bytes([
                    self.buffer[0],
                    self.buffer[1],
                    self.buffer[2],
                    self.buffer[3],
                ]) - 1;

                self.buffer.drain(0..4);

                if data_len > 0 {
                    println!("Setting Remaining Item Length: {:?}", data_len);
                    self.remaining_items = Some(data_len);
                }
            }
            (VTableFieldType::Vec(_), None) => {
                let data_len = u32::from_le_bytes([
                    self.buffer[0],
                    self.buffer[1],
                    self.buffer[2],
                    self.buffer[3],
                ]);

                self.buffer.drain(0..4);

                println!("Setting Remaining Item Length: {:?}", data_len);

                if data_len > 0 {
                    self.remaining_items = Some(data_len);
                }
            }
            // (_, Some(n)) => {
            //     println!("Decrementing remaining items.");
            //     if n == 1 {
            //         self.remaining_items = None;
            //     } else {
            //         self.remaining_items = Some(n - 1);
            //     }
            // }
            _ => {}
        }

        Ok(())
    }

    #[inline]
    pub fn set_current_field(&mut self) -> Result<&'static VTableField> {
        let field = self
            .vtable
            .struct_by_index(self.current_item_index)?
            .field_by_index(&self.current_field_index)?;

        self.current_field = Some(field);
        Ok(field)
    }

    // Return the current item or find it in the vtable based on the current item index
    #[inline]
    pub fn current_item(&mut self) -> Result<&'static VTableItem> {
        Ok(match self.current_item {
            Some(item) => item,
            _ => self.set_current_item()?,
        })
    }

    // Set the current item based on the current item index
    #[inline]
    pub fn set_current_item(&mut self) -> Result<&'static VTableItem> {
        println!("Setting current item to {:?}", self.current_item_index);
        // let field = self.current_field()?;
        let item = self.vtable.item_by_index(self.current_item_index)?;
        // self.current_item_index = field.item_index;
        self.current_item = Some(item);
        Ok(item)
    }
}

pub fn from_docbuf<'de, T>(buffer: &'de mut Vec<u8>) -> Result<T>
where
    T: Deserialize<'de> + DocBuf,
{
    let vtable = T::vtable()?;
    let mut deserializer = DocBufDeserializer::new(vtable, buffer)?;
    let t = T::deserialize(&mut deserializer)?;

    match deserializer.buffer.is_empty() {
        true => Ok(t),
        false => Err(Error::Serde("Unhandled trailing bytes".to_string())),
    }
}

impl<'de> MapAccess<'de> for &mut DocBufDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        let field = self.current_field()?;

        // Set the hash map field items to the length of the data.
        if let VTableFieldType::HashMap { .. } = &field.r#type {
            match self.remaining_items {
                None => {
                    self.remaining_items = Some(u32::from_le_bytes([
                        self.buffer[0],
                        self.buffer[1],
                        self.buffer[2],
                        0,
                    ]));

                    // drain the length from the buffer
                    self.buffer.drain(0..3);
                }
                Some(0) => {
                    self.remaining_items = None;
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
        if let VTableFieldType::HashMap { .. } = &self.current_field()?.r#type {
            // Decrement the remaining field items if it is set
            self.remaining_items.as_mut().map(|x| *x -= 1);
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
        let item = self.current_item()?;
        let field = self.current_field()?;

        println!("Next Element Seed: {field:?}");

        match &field.r#type {
            VTableFieldType::Struct(_) => {
                self.next_field()?;
            }
            VTableFieldType::Vec(_) => {
                println!("Deserialize Vec");
                self.set_remaining_items(&field.r#type)?;

                if self.remaining_items.is_none() {
                    println!("{:?}", self.has_visited);
                    return Ok(None);
                }

                self.next_field()?;
            }
            VTableFieldType::Option(_) => {}
            _ => {}
        }

        println!("Remaining Items: {:?}", self.remaining_items);

        if let Some(remaining) = self.remaining_items.as_mut() {
            if *remaining == 0 {
                self.remaining_items = None;
                return Ok(None);
            } else {
                *remaining -= 1;
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
        println!("Deserialize Any");

        let field = self.current_field()?;
        // self.optional_field.as_ref().unwrap_or()
        match &field.r#type {
            VTableFieldType::Struct(_) => {
                self.next_field()?;
                self.deserialize_map(visitor)
            }
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
            VTableFieldType::Vec(_) => self.deserialize_seq(visitor),
            VTableFieldType::HashMap { .. } => visitor.visit_map(self),
            VTableFieldType::Option(_) => self.deserialize_option(visitor),
            VTableFieldType::Uuid => unimplemented!("UUID not implemented"),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("Deserialize Bool");

        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_bool(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("Deserialize I8");

        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("Deserialize I16");

        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_i32");

        // Increment the field index
        self.next_field()?;

        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_i64");

        // Increment the field index
        self.next_field()?;

        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_i128");

        // Increment the field index
        self.next_field()?;

        visitor.visit_i128(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("Deserialize U8; Buffer: {:?}", self.buffer);
        let value = self.current_field()?.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field = self.current_field()?;
        println!("deserialize_u16");

        let value = field.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_u32");

        self.next_field()?;

        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_u64");

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
        println!("deserialize_u128");

        // Increment the field index
        self.next_field()?;

        visitor.visit_u128(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_f32");

        // Increment the field index
        self.next_field()?;

        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_f64");

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
        println!("deserialize_str");

        let value: String = field.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_str(value.as_str())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field = self.current_field()?;
        println!("deserialize_string");

        let value = field.decode(self.buffer)?;

        match field.r#type {
            // Ignore Uuid field.
            VTableFieldType::Uuid => {}
            _ => {
                self.next_field()?;
            }
        }

        visitor.visit_string(value)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let field = self.current_field()?;
        println!("deserialize_bytes");

        println!("Deserialize Bytes: {:?}", field);

        let value: Vec<u8> = field.decode(self.buffer)?;

        self.next_field()?;

        visitor.visit_bytes(value.as_slice())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self.current_field()?.decode(self.buffer)?;
        println!("deserialize_byte_buf");

        self.next_field()?;

        visitor.visit_byte_buf(value)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("Deserialize Option");
        let field = self.current_field()?;
        match field.decode_option(self.buffer)? {
            None => {
                self.next_field()?;
                return visitor.visit_none();
            }
            Some(_) => {
                self.next_field()?;

                return visitor.visit_some(self);
            }
        }
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
        println!("\n\nDeserialize Seq\n\n");
        println!("Buffer: {:?}", self.buffer);
        let field = self.current_field()?;

        println!("Field: {field:?}");

        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("deserialize_tuple");

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
        println!("Deserialize Map");

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
        println!("Deserializing Struct: {name}");

        let current_struct = self.vtable.struct_by_name(name)?;

        self.current_item_index = current_struct.item_index;
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
        let field = self.current_field()?;
        println!("deserialize_identifier");

        match &field.r#type {
            VTableFieldType::Struct(_) => {
                self.next_field()?;
                visitor.visit_str(&field.name)
            }
            VTableFieldType::HashMap { key, .. } => match key.as_ref() {
                VTableFieldType::String => {
                    // NOTE: Do not increment the field index here,
                    // as we are processing the key of the hash map.
                    let data = field.decode(self.buffer)?;
                    visitor.visit_string(data)
                }
                _ => {
                    unimplemented!("deserialize_identifier for hash map key type: {:?}", key);
                }
            },
            _ => visitor.visit_str(&field.name),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
