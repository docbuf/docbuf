use std::collections::VecDeque;

use serde::Serialize;
use tracing::debug;

use crate::vtable::*;
use crate::{
    error::Error,
    traits::{DocBuf, DocBufEncodeField},
    Result,
};

const DEFAULT_CAPACITY_MULTIPLIER: usize = 10;

#[derive(Debug)]
pub struct DocBufSerializer<'a> {
    pub vtable: &'static VTable,
    pub buffer: &'a mut Vec<u8>,
    pub current_item_index: u8,
    pub current_field_index: u8,
    pub previous_item_index: u8,
    pub current_item: Option<&'static VTableItem>,
    pub current_field: Option<&'static VTableField>,
    pub previous_item: Option<&'static VTableItem>,
    pub previous_items: VecDeque<&'static VTableItem>,
    pub offsets: VTableFieldOffsets,
}

impl<'a> DocBufSerializer<'a> {
    pub fn new(vtable: &'static VTable, buffer: &'a mut Vec<u8>) -> Self {
        // Clear the buffer before serializing
        buffer.clear();

        Self {
            vtable,
            buffer,
            current_item_index: 0,
            current_field_index: 0,
            previous_item_index: 0,
            current_item: None,
            current_field: None,
            previous_item: None,
            previous_items: VecDeque::new(),
            offsets: VTableFieldOffsets::with_capacity(
                vtable.num_items as usize * DEFAULT_CAPACITY_MULTIPLIER,
            ),
        }
    }

    // Return the current field or find it in the vtable based on the
    // current_item_index and current_field_index
    pub fn current_field(&self) -> Result<&'static VTableField> {
        Ok(match self.current_field {
            Some(field) => field,
            _ => self
                .vtable
                .struct_by_index(self.current_item_index)?
                .field_by_index(&self.current_field_index)?,
        })
    }

    pub fn encode_array_start(&mut self, num_elements: usize) -> Result<()> {
        let field = self.current_field()?;

        // println!("encode_array_start: {:?}", field);

        field.encode_array_start(num_elements, &mut self.buffer)?;

        Ok(())
    }

    // Encode the beginning of a map structure
    pub fn encode_map_start(&mut self, num_entries: usize) -> Result<()> {
        self.current_field()?
            .encode_map_start(num_entries, &mut self.buffer)?;

        Ok(())
    }
}

pub fn to_docbuf<T>(value: &T, buffer: &mut Vec<u8>) -> Result<VTableFieldOffsets>
where
    T: Serialize + DocBuf + std::fmt::Debug,
{
    let mut serializer = DocBufSerializer::new(T::vtable()?, buffer);

    value.serialize(&mut serializer)?;

    // println!("Buffer: {:?}", buffer);

    Ok(serializer.offsets)
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
        // println!("serialize_struct: {}", name);

        for vtable_item in self.vtable.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.name == name {
                        self.previous_item_index = self.current_item_index;
                        self.current_item_index = vtable_struct.item_index;
                        self.previous_item = self.current_item;

                        // Push the item to the previous items
                        if let Some(item) = self.current_item {
                            self.previous_items.push_back(item);
                        }

                        self.current_item = Some(vtable_item);
                        return Ok(self);
                    }
                }
            }
        }

        Err(Error::Serde(format!(
            "Struct {} not found in the vtable",
            name
        )))
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let offset = self.current_field()?.encode(&v, &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let offset = self.current_field()?.encode(&v, &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        unimplemented!("serialize_char")
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::F32(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::F64(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::I8(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::I16(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::I32(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::I64(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::I128(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let offset = self.current_field()?.encode(&v, &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::U8(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::U16(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::U32(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::U64(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        let offset = self
            .current_field()?
            .encode(&NumericValue::U128(v), &mut self.buffer)?;
        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        let offset = self.current_field()?.encode_none(&mut self.buffer)?;

        self.offsets.push(offset);

        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        debug!("Serializing Option::Some value");
        T::serialize(value, self)
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
        // println!("Serialize Sequence");

        self.encode_array_start(len.unwrap_or_default())?;

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        let field = self.current_field()?;

        if let VTableFieldType::Bytes = field.r#type {
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
        // // Encode the number of entries in the map
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

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: Serialize,
    {
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        key.serialize(&mut **self)?;

        value.serialize(&mut **self)?;

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
        self.set_field(name)?;

        // Serialize the field
        value.serialize(&mut **self)
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

impl<'a, 'b> DocBufSerializer<'b> {
    pub fn set_field(&'a mut self, field_name: &'static str) -> Result<()> {
        // println!("Current Item: {:?}", self.current_item);

        match self.current_item {
            Some(VTableItem::Struct(field_struct)) => {
                match field_struct.field_by_name(field_name) {
                    Ok(field) => {
                        self.current_field_index = field.index;
                        self.current_field = Some(field);

                        return Ok(());
                    }
                    Err(_) => {
                        // println!("Previous Items: {:?}", self.previous_items.len());

                        // Attempt to find the field in the previous items
                        // If the field is found, set the current item to the previous item
                        // and set the previous item to None
                        while let Some(previous_item) = self.previous_items.pop_front() {
                            // println!("Previous Item: {:?}", previous_item);
                            match previous_item {
                                VTableItem::Struct(field_struct) => {
                                    match field_struct.field_by_name(field_name) {
                                        Ok(field) => {
                                            self.current_item = Some(previous_item);
                                            self.current_item_index = self.previous_item_index;
                                            self.previous_item = None;

                                            self.current_field_index = field.index;
                                            self.current_field = Some(field);

                                            return Ok(());
                                        }
                                        Err(_) => {
                                            self.previous_items.push_back(previous_item);
                                        }
                                    }
                                }
                            }
                        }

                        return Err(Error::Serde(format!("Field not found: {}", field_name)));
                    }
                }
            }
            _ => match self
                .vtable
                .struct_by_index(self.current_item_index)?
                .field_by_name(field_name)
            {
                Ok(field) => {
                    self.current_field_index = field.index;
                    self.current_field = Some(field);

                    return Ok(());
                }
                Err(_) => {
                    match self
                        .vtable
                        .struct_by_index(self.previous_item_index)?
                        .field_by_name(field_name)
                    {
                        Ok(field) => {
                            // Reset the current item to the parent struct
                            self.current_item = self.previous_item;
                            self.current_item_index = self.previous_item_index;
                            self.previous_item = None;

                            self.current_field_index = field.index;
                            self.current_field = Some(field);

                            // Return the field
                            return Ok(());
                        }
                        Err(_) => {
                            return Err(Error::Serde(format!("Field not found: {}", field_name)));
                        }
                    }
                }
            },
        };
    }
}
