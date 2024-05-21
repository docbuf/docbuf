use std::cmp::Ordering;

use crate::Error;

use docbuf_core::{
    traits::DocBufDecodeField,
    vtable::{VTable, VTableField, VTableFieldOffsetIndex, VTableFieldOffsets, VTableFieldType},
};

#[derive(Debug, Clone)]
pub struct Predicates {
    and: Vec<Predicate>,
    or: Vec<Predicate>,
}

impl From<Predicate> for Predicates {
    fn from(predicate: Predicate) -> Self {
        Self::from(predicate)
    }
}

impl Predicates {
    pub fn from(predicate: Predicate) -> Self {
        Self {
            and: vec![predicate],
            or: Vec::new(),
        }
    }

    pub fn new() -> Self {
        Self {
            and: Vec::new(),
            or: Vec::new(),
        }
    }

    // pub fn iter(&self) -> impl Iterator<Item = &Predicate> {
    //     self.0.iter()
    // }

    pub fn and(mut self, predicate: Predicate) -> Self {
        self.and.push(predicate);

        self
    }

    pub fn or(mut self, predicate: Predicate) -> Self {
        self.or.push(predicate);

        self
    }

    pub fn eval(
        &self,
        vtable: &VTable,
        offsets: &VTableFieldOffsets,
        buffer: &[u8],
    ) -> Result<bool, Error> {
        let mut result = true;

        for predicate in self.and.iter() {
            if !predicate.eval(vtable, offsets, buffer)? {
                result = false;
            }
        }

        for predicate in self.or.iter() {
            if predicate.eval(vtable, offsets, buffer)? {
                result = true;
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct Predicate {
    /// The field offset to compare against.
    offset: VTableFieldOffsetIndex,
    /// The value to compare against.
    value: Vec<u8>,
    /// The type of predicate.
    order: Ordering,
}

impl AsRef<[u8]> for Predicate {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

impl Predicate {
    pub fn new(
        field: &VTableField,
        compare_value: impl Into<Vec<u8>>,
        order: Ordering,
    ) -> Predicate {
        let offset = field.offset_index();

        Self {
            offset,
            value: compare_value.into(),
            order,
        }
    }

    pub fn offset(&self) -> VTableFieldOffsetIndex {
        self.offset
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn order(&self) -> Ordering {
        self.order
    }

    pub fn eval(
        &self,
        vtable: &VTable,
        offsets: &VTableFieldOffsets,
        buffer: &[u8],
    ) -> Result<bool, Error> {
        match offsets.offset(self.offset()) {
            None => return Ok(false),
            Some(offset) => {
                let data = &buffer[offset.range()];

                if self.order() == Ordering::Equal {
                    return Ok(self.value() == data);
                }

                let field = vtable.get_field_by_offset_index(self.offset())?;

                match field.r#type {
                    VTableFieldType::I8 => {
                        let value: i8 = field.decode(&mut self.value().to_vec())?;
                        let field_value: i8 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::I16 => {
                        let value: i16 = field.decode(&mut self.value().to_vec())?;
                        let field_value: i16 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::I32 => {
                        let value: i32 = field.decode(&mut self.value().to_vec())?;
                        let field_value: i32 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::I64 => {
                        let value: i64 = field.decode(&mut self.value().to_vec())?;
                        let field_value: i64 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }

                    VTableFieldType::I128 => {
                        let value: i128 = field.decode(&mut self.value().to_vec())?;
                        let field_value: i128 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::ISIZE => {
                        let value: isize = field.decode(&mut self.value().to_vec())?;
                        let field_value: isize = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::U8 => {
                        let value: u8 = field.decode(&mut self.value().to_vec())?;
                        let field_value: u8 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::U16 => {
                        let value: u16 = field.decode(&mut self.value().to_vec())?;
                        let field_value: u16 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::U32 => {
                        let value: u32 = field.decode(&mut self.value().to_vec())?;
                        let field_value: u32 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::U64 => {
                        let value: u64 = field.decode(&mut self.value().to_vec())?;
                        let field_value: u64 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::U128 => {
                        let value: u128 = field.decode(&mut self.value().to_vec())?;
                        let field_value: u128 = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::USIZE => {
                        let value: usize = field.decode(&mut self.value().to_vec())?;
                        let field_value: usize = field.decode(&mut data.to_vec())?;

                        return Ok(value.cmp(&field_value) == self.order());
                    }
                    VTableFieldType::F32 => {
                        let value: f32 = field.decode(&mut self.value().to_vec())?;
                        let field_value: f32 = field.decode(&mut data.to_vec())?;

                        return Ok(value.partial_cmp(&field_value) == Some(self.order()));
                    }
                    VTableFieldType::F64 => {
                        let value: f64 = field.decode(&mut self.value().to_vec())?;
                        let field_value: f64 = field.decode(&mut data.to_vec())?;

                        return Ok(value.partial_cmp(&field_value) == Some(self.order()));
                    }
                    VTableFieldType::String
                    | VTableFieldType::Str
                    | VTableFieldType::Bytes
                    | VTableFieldType::Vec
                    | VTableFieldType::Bool
                    | VTableFieldType::HashMap { .. }
                    | VTableFieldType::Uuid
                    | VTableFieldType::Struct(_) => {
                        return Ok(self.value().cmp(&data) == self.order());
                    }
                }
            }
        }
    }
}
