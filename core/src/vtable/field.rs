#![allow(unused_imports)]

mod decode;
mod encode;
mod numeric;
mod offset;
mod rules;
#[cfg(feature = "validate")]
mod validate;

use super::*;

// Re-export Field Implementations
pub use decode::*;
pub use encode::*;
pub use numeric::*;
pub use offset::*;
pub use rules::*;
#[cfg(feature = "validate")]
pub use validate::*;

use crate::traits::{DocBufDecodeField, DocBufEncodeField, DocBufValidateField};

use std::ops::{Deref, Range};
use std::{cmp::Ordering, str::FromStr};

use serde_derive::{Deserialize, Serialize};

// Number of bytes in a gigabyte as a usize
pub const GIGABYTE: usize = 1024 * 1024 * 1024;

// Maximum size of a field in a struct
pub const MAX_FIELD_SIZE: usize = GIGABYTE;

// Maximum number of map entries
pub const MAX_MAP_ENTRIES: usize = 256 * 256 * 256;

// Default field length encoded as 4 le bytes
pub const DEFAULT_FIELD_LENGTH_LE_BYTES: usize = 4;

pub type VTableFieldIndex = u8;
pub type VTableFieldName = String; //  = &'a str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VTableField {
    /// The index of the vtable item this field belongs to
    pub item_index: VTableItemIndex,
    /// The type of the field
    pub r#type: VTableFieldType,
    pub index: VTableFieldIndex,
    pub name: VTableFieldName,
    pub rules: VTableFieldRules,
}

impl VTableField {
    pub fn new(
        item_index: VTableItemIndex,
        r#type: VTableFieldType,
        index: VTableFieldIndex,
        name: VTableFieldName,
        rules: VTableFieldRules,
    ) -> Self {
        Self {
            item_index,
            r#type,
            index,
            name,
            rules,
        }
    }

    pub fn encode_array_start(
        &self,
        num_elements: usize,
        output: &mut Vec<u8>,
    ) -> Result<(), Error> {
        // Check if the num elements exceeds the maximum allowed.
        if num_elements >= MAX_FIELD_SIZE {
            return Err(Error::ArrayElementsExceedsMax(num_elements));
        }

        // Only encode the first three bytes
        output.extend_from_slice(&(num_elements as u32).to_le_bytes());

        Ok(())
    }

    pub fn encode_map_start(&self, num_entries: usize, output: &mut Vec<u8>) -> Result<(), Error> {
        // Check if the num entries exceeds the maximum allowed.
        if num_entries >= MAX_MAP_ENTRIES {
            return Err(Error::MapEntriesExceedsMax(num_entries));
        }

        output.extend_from_slice(&(num_entries as u32).to_le_bytes());

        Ok(())
    }

    #[inline]
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        // Push the item index
        buffer.push(self.item_index);

        // Write the field type
        buffer.push(self.r#type.clone().into());

        // If the field type is a struct, write the struct name.
        match &self.r#type {
            VTableFieldType::Struct(name) => {
                let name_bytes = name.as_bytes();
                buffer.push(name_bytes.len() as u8);
                buffer.extend_from_slice(name_bytes);
            }
            VTableFieldType::HashMap { key, value } => {
                buffer.push(key.deref().to_owned().into());
                buffer.push(value.deref().to_owned().into());
            }
            _ => (),
        }

        // Write the field index
        buffer.push(self.index);

        // Write the field name
        let name_bytes = self.name.as_bytes();

        buffer.push(name_bytes.len() as u8);
        buffer.extend_from_slice(name_bytes);

        // Write the field rules
        self.rules.write_to_buffer(buffer)?;

        Ok(())
    }

    #[inline]
    pub fn read_from_buffer(buffer: &mut Vec<u8>) -> Result<Self, Error> {
        // Read the item index
        let item_index = buffer.remove(0);

        // Read the field type
        let mut r#type = VTableFieldType::try_from(buffer.remove(0))?;

        // If the type is a struct, read the struct name
        match r#type {
            VTableFieldType::Struct(_) => {
                let name_len = buffer.remove(0);
                let name = buffer.drain(0..name_len as usize).collect::<Vec<u8>>();
                let name = String::from_utf8(name)?;

                r#type = VTableFieldType::Struct(name);
            }
            VTableFieldType::HashMap { .. } => {
                let key = Box::new(VTableFieldType::try_from(buffer.remove(0))?);
                let value = Box::new(VTableFieldType::try_from(buffer.remove(0))?);

                r#type = VTableFieldType::HashMap { key, value };
            }
            _ => (),
        }

        // Read the field index
        let index = buffer.remove(0);

        // Read the field name
        let name_len = buffer.remove(0);
        let name = buffer.drain(0..name_len as usize).collect::<Vec<u8>>();

        let name = String::from_utf8(name)?;

        // Read the field rules
        let rules = VTableFieldRules::read_from_buffer(buffer)?;

        Ok(Self {
            item_index,
            r#type,
            index,
            name,
            rules,
        })
    }

    #[inline]
    pub fn offset_index(&self) -> VTableFieldOffsetIndex {
        (self.item_index, self.index)
    }
}

impl std::fmt::Display for VTableField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {} {:?}",
            self.r#type, self.index, self.name, self.rules
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VTableFields(pub Vec<VTableField>);

impl Default for VTableFields {
    fn default() -> Self {
        Self::new()
    }
}

impl VTableFields {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::with_capacity(256))
    }

    // Add a field to the vtable fields
    #[inline]
    pub fn add_field(&mut self, field: VTableField) {
        self.0.push(field);
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, VTableField> {
        self.0.iter()
    }

    // Inner values
    #[inline]
    pub fn inner(&self) -> &Vec<VTableField> {
        &self.0
    }

    // Inner values mutable
    #[inline]
    pub fn inner_mut(&mut self) -> &mut Vec<VTableField> {
        &mut self.0
    }

    // Returns the length of items
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Find a field by its name
    #[inline]
    pub fn find_field_by_name(&self, name: &str) -> Option<&VTableField> {
        self.0.iter().find(|field| field.name == name)
    }

    #[inline]
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        // Write each field
        for field in self.0.iter() {
            field.write_to_buffer(buffer)?;
        }

        Ok(())
    }

    /// Find a field by its item index and index
    #[inline]
    pub fn find_field_by_index(&self, item_index: u8, index: u8) -> Option<&VTableField> {
        self.0
            .iter()
            .find(|field| field.item_index == item_index && field.index == index)
    }
}

impl PartialEq for VTableFields {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq for VTableField {
    fn eq(&self, other: &Self) -> bool {
        self.item_index == other.item_index
            && self.r#type == other.r#type
            && self.index == other.index
            && self.name == other.name
            && self.rules == other.rules
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum VTableFieldType {
    U8,
    U16,
    U32,
    U64,
    U128,
    USIZE,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISIZE,
    F32,
    F64,
    String,
    Str,
    Bytes,
    Bool,
    Struct(StructName),
    Option(Box<VTableFieldType>),
    Vec(Box<VTableFieldType>),
    HashMap {
        key: Box<VTableFieldType>,
        value: Box<VTableFieldType>,
    },
    Uuid,
}

impl std::fmt::Display for VTableFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VTableFieldType::U8 => write!(f, "u8"),
            VTableFieldType::U16 => write!(f, "u16"),
            VTableFieldType::U32 => write!(f, "u32"),
            VTableFieldType::U64 => write!(f, "u64"),
            VTableFieldType::U128 => write!(f, "u128"),
            VTableFieldType::USIZE => write!(f, "usize"),
            VTableFieldType::I8 => write!(f, "i8"),
            VTableFieldType::I16 => write!(f, "i16"),
            VTableFieldType::I32 => write!(f, "i32"),
            VTableFieldType::I64 => write!(f, "i64"),
            VTableFieldType::I128 => write!(f, "i128"),
            VTableFieldType::ISIZE => write!(f, "isize"),
            VTableFieldType::F32 => write!(f, "f32"),
            VTableFieldType::F64 => write!(f, "f64"),
            VTableFieldType::String => write!(f, "String"),
            VTableFieldType::Str => write!(f, "&str"),
            VTableFieldType::Vec(t) => write!(f, "Vec<{}>", t),
            VTableFieldType::Bytes => write!(f, "Bytes"),
            VTableFieldType::Bool => write!(f, "bool"),
            VTableFieldType::Struct(s) => write!(f, "{}", s),
            VTableFieldType::HashMap { key, value } => {
                write!(f, "HashMap<{}, {}>", key, value)
            }
            VTableFieldType::Uuid => write!(f, "Uuid"),
            VTableFieldType::Option(t) => write!(f, "Option<{}>", t),
        }
    }
}

impl VTableFieldType {
    pub fn is_struct(r#type: impl TryInto<Self>) -> Option<StructName> {
        match r#type.try_into() {
            Ok(VTableFieldType::Struct(name)) => Some(name),
            Ok(VTableFieldType::Option(opt)) => Self::is_struct(*opt),
            Ok(VTableFieldType::Vec(t)) => Self::is_struct(*t),
            _ => None,
        }
    }

    pub(crate) fn parse_hashmap_types(input: &str) -> VTableFieldType {
        let mut types = input.split('<');
        types = types
            .nth(1)
            .map(|s| s.trim_end_matches('>'))
            .expect("failed to parse hash map key,pair types")
            .split(',');
        let key = types
            .nth(0)
            .map(|k| k.trim())
            .expect("failed to parse hash map key type");
        let value = types
            .nth(0)
            .map(|s| s.trim())
            .expect("failed to parse hash map value type");

        VTableFieldType::HashMap {
            key: Box::new(VTableFieldType::from(key)),
            value: Box::new(VTableFieldType::from(value)),
        }
    }
}

impl From<&str> for VTableFieldType {
    fn from(s: &str) -> Self {
        // println!("field type: {s}");

        match s {
            "u8" => VTableFieldType::U8,
            "u16" => VTableFieldType::U16,
            "u32" => VTableFieldType::U32,
            "u64" => VTableFieldType::U64,
            "u128" => VTableFieldType::U128,
            "usize" => VTableFieldType::USIZE,
            "i8" => VTableFieldType::I8,
            "i16" => VTableFieldType::I16,
            "i32" => VTableFieldType::I32,
            "i64" => VTableFieldType::I64,
            "i128" => VTableFieldType::I128,
            "isize" => VTableFieldType::ISIZE,
            "f32" => VTableFieldType::F32,
            "f64" => VTableFieldType::F64,
            "String" => VTableFieldType::String,
            // "[u8; 32]" => VTableFieldType::Bytes,
            "Vec < u8 >" => VTableFieldType::Bytes,
            s if s.contains("str") => VTableFieldType::Str,
            // s if s.contains("[u8]") => VTableFieldType::Bytes,
            // s if s.contains("[u8; ") => VTableFieldType::Bytes,
            s if s.contains("[u8") => VTableFieldType::Bytes,
            s if s.contains("Uuid") => VTableFieldType::Uuid,
            s if s.contains("Option < Vec < u8 ") => {
                VTableFieldType::Option(Box::new(VTableFieldType::Bytes))
            }
            s if s.contains("Option < Vec < ") => {
                // println!("Handle Option Vec: {s}");
                let t = s.trim_start_matches("Option < ").trim_end_matches(" >");
                let t = t.trim_start_matches("Vec < ").trim_end_matches(" >");
                VTableFieldType::Option(Box::new(VTableFieldType::Vec(Box::new(
                    VTableFieldType::from(t),
                ))))
            }
            s if s.contains("Vec") => {
                let t = s.trim_start_matches("Vec < ").trim_end_matches(" >");
                VTableFieldType::Vec(Box::new(VTableFieldType::from(t)))
            }
            s if s.contains("Option") => {
                let t = s.trim_start_matches("Option < ").trim_end_matches(" >");
                VTableFieldType::Option(Box::new(VTableFieldType::from(t)))
            }
            "bool" => VTableFieldType::Bool,
            s if s.contains("HashMap") => VTableFieldType::parse_hashmap_types(s),
            s => VTableFieldType::Struct(s.to_owned()),
        }
    }
}

impl TryFrom<u8> for VTableFieldType {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self, Error> {
        let r#type = match byte {
            0 => VTableFieldType::U8,
            1 => VTableFieldType::U16,
            2 => VTableFieldType::U32,
            3 => VTableFieldType::U64,
            4 => VTableFieldType::U128,
            5 => VTableFieldType::USIZE,
            6 => VTableFieldType::I8,
            7 => VTableFieldType::I16,
            8 => VTableFieldType::I32,
            9 => VTableFieldType::I64,
            10 => VTableFieldType::I128,
            11 => VTableFieldType::ISIZE,
            12 => VTableFieldType::F32,
            13 => VTableFieldType::F64,
            14 => VTableFieldType::String,
            15 => VTableFieldType::Str,
            16 => VTableFieldType::Vec(Box::new(VTableFieldType::U8)),
            17 => VTableFieldType::Bytes,
            18 => VTableFieldType::Bool,
            19 => VTableFieldType::Struct(String::new()),
            20 => VTableFieldType::HashMap {
                key: Box::new(VTableFieldType::U8),
                value: Box::new(VTableFieldType::U8),
            },
            21 => VTableFieldType::Uuid,
            22 => VTableFieldType::Option(Box::new(VTableFieldType::U8)),
            _ => return Err(Error::UnknownFieldType(byte)),
        };

        Ok(r#type)
    }
}

impl Into<u8> for VTableFieldType {
    fn into(self) -> u8 {
        match self {
            VTableFieldType::U8 => 0,
            VTableFieldType::U16 => 1,
            VTableFieldType::U32 => 2,
            VTableFieldType::U64 => 3,
            VTableFieldType::U128 => 4,
            VTableFieldType::USIZE => 5,
            VTableFieldType::I8 => 6,
            VTableFieldType::I16 => 7,
            VTableFieldType::I32 => 8,
            VTableFieldType::I64 => 9,
            VTableFieldType::I128 => 10,
            VTableFieldType::ISIZE => 11,
            VTableFieldType::F32 => 12,
            VTableFieldType::F64 => 13,
            VTableFieldType::String => 14,
            VTableFieldType::Str => 15,
            VTableFieldType::Vec(_) => 16,
            VTableFieldType::Bytes => 17,
            VTableFieldType::Bool => 18,
            VTableFieldType::Struct(_) => 19,
            VTableFieldType::HashMap { .. } => 20, //
            VTableFieldType::Uuid => 21,
            VTableFieldType::Option(_) => 22,
        }
    }
}

impl PartialEq for VTableFieldType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VTableFieldType::U8, VTableFieldType::U8)
            | (VTableFieldType::U16, VTableFieldType::U16)
            | (VTableFieldType::U32, VTableFieldType::U32)
            | (VTableFieldType::U64, VTableFieldType::U64)
            | (VTableFieldType::U128, VTableFieldType::U128)
            | (VTableFieldType::USIZE, VTableFieldType::USIZE)
            | (VTableFieldType::I8, VTableFieldType::I8)
            | (VTableFieldType::I16, VTableFieldType::I16)
            | (VTableFieldType::I32, VTableFieldType::I32)
            | (VTableFieldType::I64, VTableFieldType::I64)
            | (VTableFieldType::I128, VTableFieldType::I128)
            | (VTableFieldType::ISIZE, VTableFieldType::ISIZE)
            | (VTableFieldType::F32, VTableFieldType::F32)
            | (VTableFieldType::F64, VTableFieldType::F64)
            | (VTableFieldType::String, VTableFieldType::String)
            | (VTableFieldType::Str, VTableFieldType::Str)
            | (VTableFieldType::Bytes, VTableFieldType::Bytes)
            | (VTableFieldType::Bool, VTableFieldType::Bool)
            | (VTableFieldType::Uuid, VTableFieldType::Uuid) => true,
            (VTableFieldType::Struct(s1), VTableFieldType::Struct(s2)) => s1 == s2,
            (
                VTableFieldType::HashMap { key: k1, value: v1 },
                VTableFieldType::HashMap { key: k2, value: v2 },
            ) => k1 == k2 && v1 == v2,
            (VTableFieldType::Option(o1), VTableFieldType::Option(o2)) => o1 == o2,
            (VTableFieldType::Vec(v1), VTableFieldType::Vec(v2)) => v1 == v2,
            _ => false,
        }
    }
}
