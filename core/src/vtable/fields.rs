use std::ops::Range;
use std::{cmp::Ordering, str::FromStr};

use crate::traits::DocBufDecodeField;

pub use super::*;

#[cfg(feature = "regex")]
use crate::validate::regex::Regex;

// Number of bytes in a gigabyte as a usize
pub const GIGABYTE: usize = 1024 * 1024 * 1024;

// Maximum size of a field in a struct
pub const MAX_FIELD_SIZE: usize = GIGABYTE;

// Maximum number of map entries
pub const MAX_MAP_ENTRIES: usize = 256 * 256 * 256;

// Default field length encoded as 4 le bytes
pub const DEFAULT_FIELD_LENGTH_LE_BYTES: usize = 4;

pub const U8_MAX: usize = u8::MAX as usize;
pub const U16_MAX: usize = u16::MAX as usize;
pub const U32_MAX: usize = u32::MAX as usize;
pub const U64_MAX: usize = u64::MAX as usize;

// Field Offset range of the resulting document buffer bytes
pub type VTableFieldOffset = (VTableItemIndex, VTableFieldIndex, Range<usize>);
pub type VTableFieldOffsets = Vec<VTableFieldOffset>;
pub type VTableFieldIndex = u8;
pub type VTableFieldName<'a> = &'a str;

#[derive(Debug, Clone)]
pub struct VTableField<'a> {
    /// The index of the vtable item this field belongs to
    pub item_index: VTableItemIndex,
    /// The type of the field
    pub field_type: VTableFieldType<'a>,
    pub field_index: VTableFieldIndex,
    pub field_name: VTableFieldName<'a>,
    pub field_rules: VTableFieldRules,
}

impl<'a> VTableField<'a> {
    pub fn new(
        item_index: VTableItemIndex,
        field_type: VTableFieldType<'a>,
        field_index: VTableFieldIndex,
        field_name: VTableFieldName<'a>,
        field_rules: VTableFieldRules,
    ) -> Self {
        // println!("Field Rules: {:?}", field_rules);

        Self {
            item_index,
            field_type,
            field_index,
            field_name,
            field_rules,
        }
    }

    pub fn encode_array_start(
        &self,
        num_elements: usize,
        output: &mut Vec<u8>,
    ) -> Result<(), Error> {
        // println!("Num elements: {}", num_elements);

        // Check if the num elements exceeds the maximum allowed.
        if num_elements >= MAX_FIELD_SIZE {
            return Err(Error::ArrayElementsExceedsMax(num_elements));
        }

        // Only encode the first three bytes
        output.extend_from_slice(&(num_elements as u32).to_le_bytes());

        // println!("encode_array_start OUTPUT: {:?}", output);

        Ok(())
    }

    pub fn encode_map_start(&self, num_entries: usize, output: &mut Vec<u8>) -> Result<(), Error> {
        // println!("Num entries: {}", num_entries);

        // Check if the num entries exceeds the maximum allowed.
        if num_entries >= MAX_MAP_ENTRIES {
            return Err(Error::MapEntriesExceedsMax(num_entries));
        }

        // Only encode the first three bytes
        output.extend_from_slice(&(num_entries as u32).to_le_bytes()[0..3]);

        // println!("encode_map_start OUTPUT: {:?}", output);

        Ok(())
    }

    pub fn encode_str(&self, field_data: &str, output: &mut Vec<u8>) -> Result<(), Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.validate_str(field_data)?;

        match &self.field_type {
            VTableFieldType::String => {
                // prepend length to the field data
                output.extend_from_slice(&(field_data.len() as u32).to_le_bytes());
            }
            VTableFieldType::HashMap { key, value } => {
                // prepend length to the field data
                output.extend_from_slice(&(field_data.len() as u32).to_le_bytes());
            }
            _ => {}
        };

        output.extend_from_slice(field_data.as_bytes());

        // println!("encode_str OUTPUT: {:?}", output);

        Ok(())
    }

    pub fn encode_bytes(&self, field_data: &[u8], output: &mut Vec<u8>) -> Result<(), Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.validate_bytes(field_data)?;

        // prepend length to the field data
        output.extend_from_slice(&(field_data.len() as u32).to_le_bytes());

        output.extend_from_slice(field_data);

        // println!("encode_bytes OUTPUT: {:?}", output);

        Ok(())
    }

    pub fn encode_bool(&self, field_data: bool, output: &mut Vec<u8>) -> Result<(), Error> {
        // // Ensure the field data corresponds to the field rules
        // #[cfg(feature = "validate")]
        // self.validate_bool(field_data)?;

        // Encode the field data
        output.push(if field_data { 1 } else { 0 });

        Ok(())
    }

    pub fn encode_numeric_value(
        &self,
        field_data: NumericValue,
        output: &mut Vec<u8>,
    ) -> Result<(), Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.validate_numeric_value(&field_data)?;

        // Encode the field data
        field_data.encode(output)?;

        Ok(())
    }

    /// Decode a field from the bytes, assuming the field is the next field in the bytes.
    ///
    /// Consumes the incoming bytes, returning only the span of field data bytes.
    // pub fn decode(&self, bytes: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
    //     Ok(match &self.field_type {
    //         VTableFieldType::String | VTableFieldType::Bytes => {
    //             self.decode_variable_length(bytes)?
    //         }
    //         VTableFieldType::U8 | VTableFieldType::I8 => {
    //             let field_data = bytes.drain(0..1);

    //             field_data.collect()
    //         }
    //         VTableFieldType::U16 | VTableFieldType::I16 => {
    //             let field_data = bytes.drain(0..2);

    //             field_data.collect()
    //         }
    //         VTableFieldType::U32 | VTableFieldType::F32 | VTableFieldType::I32 => {
    //             let field_data = bytes.drain(0..4);

    //             field_data.collect()
    //         }
    //         VTableFieldType::U64
    //         | VTableFieldType::F64
    //         | VTableFieldType::USIZE
    //         | VTableFieldType::I64
    //         | VTableFieldType::ISIZE => {
    //             let field_data = bytes.drain(0..8);

    //             field_data.collect()
    //         }
    //         VTableFieldType::U128 | VTableFieldType::I128 => {
    //             let field_data = bytes.drain(0..16);

    //             field_data.collect()
    //         }

    //         VTableFieldType::Struct(_) => Vec::new(),
    //         // VTableFieldType::HashMap { key, value } => self.decode_map_data(key, value, bytes)?,
    //         VTableFieldType::Bool => {
    //             let field_data = bytes.drain(0..1);

    //             field_data.collect()
    //         }
    //         _ => {
    //             unimplemented!("Decode Field Type: {:#?}", self.field_type);
    //         }
    //     })
    // }

    pub fn decode_map_data(
        &self,
        key: &Box<VTableFieldType>,
        value: &Box<VTableFieldType>,
        bytes: &mut Vec<u8>,
    ) -> Result<Vec<u8>, Error> {
        // println!("Decoding Map Data");

        let mut output = Vec::with_capacity(1024);

        let num_entries = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]);

        if num_entries == 0 {
            return Ok(output);
        }

        // Remove the first three bytes
        bytes.drain(0..3);

        // println!("Num Entries: {}", num_entries);

        // multiply the entries by 2 to account for the key and value pairs
        for _ in 0..num_entries * 2 {
            let length = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

            // Remove the encoded length from the bytes
            // bytes.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

            // Remove the field data from the bytes and return it
            output.extend_from_slice(
                bytes
                    .drain(0..length + DEFAULT_FIELD_LENGTH_LE_BYTES)
                    .as_slice(),
            );

            // println!("Remaing Bytes: {:?}", bytes);
        }

        // println!("Output: {:?}", output);

        Ok(output)
    }

    // Decodes the field data from the bytes input.
    // Returns the raw field data and removes the field data from the bytes, including its length.
    pub fn decode_variable_length(&self, bytes: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        // println!("Decoding Variable length bytes: {:?}", bytes);

        let length = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

        // Remove the encoded length from the bytes
        bytes.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

        // Remove the field data from the bytes and return it
        let field_data = bytes.drain(0..length);

        // println!("Field Data: {:?}", field_data);

        Ok(field_data.collect())
    }

    pub fn validate_bytes(&self, data: &[u8]) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        match self.field_type {
            VTableFieldType::Bytes => {
                // Check for string length rules
                self.field_rules.check_data_length_field_rules(data.len())?;
            }
            _ => {
                return Err(Error::InvalidValidationType(format!(
                    "Invalid validation type for bytes field: {:#?}",
                    self.field_type
                )))
            }
        }

        Ok(())
    }

    pub fn validate_str(&self, data: &str) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        match self.field_type {
            VTableFieldType::String => {
                // Check for string length rules
                self.field_rules.check_data_length_field_rules(data.len())?;

                // Check for regex rules
                #[cfg(feature = "regex")]
                self.field_rules.check_regex_field_rules(data)?;
            }
            _ => {
                return Err(Error::InvalidValidationType(format!(
                    "Invalid validation type for string field: {:#?}",
                    self.field_type
                )))
            }
        }

        Ok(())
    }

    pub fn validate_numeric_value(&self, value: &NumericValue) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        self.field_rules.check_numeric_value(value)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VTableFields<'a>(pub Vec<VTableField<'a>>);

impl<'a> VTableFields<'a> {
    pub fn new() -> Self {
        Self(Vec::with_capacity(256))
    }

    // Add a field to the vtable fields
    pub fn add_field(&mut self, field: VTableField<'a>) {
        self.0.push(field);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, VTableField<'a>> {
        self.0.iter()
    }

    // Inner values
    pub fn inner(&self) -> &Vec<VTableField<'a>> {
        &self.0
    }

    // Inner values mutable
    pub fn inner_mut(&mut self) -> &mut Vec<VTableField<'a>> {
        &mut self.0
    }

    // Returns the length of items
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Find a field by its name
    pub fn find_field_by_name(&self, field_name: &str) -> Option<&VTableField<'a>> {
        self.0.iter().find(|field| field.field_name == field_name)
    }
}

/// Value enum for the field type
#[derive(Debug, Clone)]
pub enum NumericValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    USIZE(usize),
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    ISIZE(isize),
}

impl NumericValue {
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), Error> {
        match self {
            NumericValue::U8(value) => {
                output.push(*value);
            }
            NumericValue::U16(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::U32(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::U64(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::U128(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::USIZE(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::F32(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::F64(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I8(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I16(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I32(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I64(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I128(value) => output.extend_from_slice(&value.to_le_bytes()),
            NumericValue::ISIZE(value) => output.extend_from_slice(&value.to_le_bytes()),
        };

        Ok(())
    }
}

impl Into<NumericValue> for u8 {
    fn into(self) -> NumericValue {
        NumericValue::U8(self)
    }
}

impl Into<NumericValue> for u16 {
    fn into(self) -> NumericValue {
        NumericValue::U16(self)
    }
}

impl Into<NumericValue> for u32 {
    fn into(self) -> NumericValue {
        NumericValue::U32(self)
    }
}

impl Into<NumericValue> for u64 {
    fn into(self) -> NumericValue {
        NumericValue::U64(self)
    }
}

impl Into<NumericValue> for u128 {
    fn into(self) -> NumericValue {
        NumericValue::U128(self)
    }
}

impl Into<NumericValue> for usize {
    fn into(self) -> NumericValue {
        NumericValue::USIZE(self)
    }
}

impl Into<NumericValue> for f32 {
    fn into(self) -> NumericValue {
        NumericValue::F32(self)
    }
}

impl Into<NumericValue> for f64 {
    fn into(self) -> NumericValue {
        NumericValue::F64(self)
    }
}

impl Into<NumericValue> for i8 {
    fn into(self) -> NumericValue {
        NumericValue::I8(self)
    }
}

impl Into<NumericValue> for i16 {
    fn into(self) -> NumericValue {
        NumericValue::I16(self)
    }
}

impl Into<NumericValue> for i32 {
    fn into(self) -> NumericValue {
        NumericValue::I32(self)
    }
}

impl Into<NumericValue> for i64 {
    fn into(self) -> NumericValue {
        NumericValue::I64(self)
    }
}

impl Into<NumericValue> for i128 {
    fn into(self) -> NumericValue {
        NumericValue::I128(self)
    }
}

impl Into<NumericValue> for isize {
    fn into(self) -> NumericValue {
        NumericValue::ISIZE(self)
    }
}

/// Implement Eq for NumericValue
impl Eq for NumericValue {}

/// Implement PartialEq for NumericValue
impl PartialEq for NumericValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NumericValue::U8(a), NumericValue::U8(b)) => a.eq(b),
            (NumericValue::U16(a), NumericValue::U16(b)) => a.eq(b),
            (NumericValue::U32(a), NumericValue::U32(b)) => a.eq(b),
            (NumericValue::U64(a), NumericValue::U64(b)) => a.eq(b),
            (NumericValue::U128(a), NumericValue::U128(b)) => a.eq(b),
            (NumericValue::USIZE(a), NumericValue::USIZE(b)) => a.eq(b),
            (NumericValue::F32(a), NumericValue::F32(b)) => a.eq(b),
            (NumericValue::F64(a), NumericValue::F64(b)) => a.eq(b),
            (NumericValue::I8(a), NumericValue::I8(b)) => a.eq(b),
            (NumericValue::I16(a), NumericValue::I16(b)) => a.eq(b),
            (NumericValue::I32(a), NumericValue::I32(b)) => a.eq(b),
            (NumericValue::I64(a), NumericValue::I64(b)) => a.eq(b),
            (NumericValue::I128(a), NumericValue::I128(b)) => a.eq(b),
            (NumericValue::ISIZE(a), NumericValue::ISIZE(b)) => a.eq(b),
            _ => false,
        }
    }
}

/// Implement Partial Ord for NumericValue
impl PartialOrd for NumericValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (NumericValue::U8(a), NumericValue::U8(b)) => a.partial_cmp(b),
            (NumericValue::U16(a), NumericValue::U16(b)) => a.partial_cmp(b),
            (NumericValue::U32(a), NumericValue::U32(b)) => a.partial_cmp(b),
            (NumericValue::U64(a), NumericValue::U64(b)) => a.partial_cmp(b),
            (NumericValue::U128(a), NumericValue::U128(b)) => a.partial_cmp(b),
            (NumericValue::USIZE(a), NumericValue::USIZE(b)) => a.partial_cmp(b),
            (NumericValue::F32(a), NumericValue::F32(b)) => a.partial_cmp(b),
            (NumericValue::F64(a), NumericValue::F64(b)) => a.partial_cmp(b),
            (NumericValue::I8(a), NumericValue::I8(b)) => a.partial_cmp(b),
            (NumericValue::I16(a), NumericValue::I16(b)) => a.partial_cmp(b),
            (NumericValue::I32(a), NumericValue::I32(b)) => a.partial_cmp(b),
            (NumericValue::I64(a), NumericValue::I64(b)) => a.partial_cmp(b),
            (NumericValue::I128(a), NumericValue::I128(b)) => a.partial_cmp(b),
            (NumericValue::ISIZE(a), NumericValue::ISIZE(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

/// Optional rules for a field
#[derive(Debug, Clone)]
pub struct VTableFieldRules {
    pub ignore: bool,
    pub max_value: Option<NumericValue>,
    pub min_value: Option<NumericValue>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    // An absolute length
    pub length: Option<usize>,
    #[cfg(feature = "regex")]
    pub regex: Option<Regex>,
    pub sign: bool,
}

impl VTableFieldRules {
    pub fn new() -> Self {
        Self {
            ignore: false,
            max_value: None,
            min_value: None,
            max_length: None,
            min_length: None,
            length: None,
            regex: None,
            sign: false,
        }
    }

    // Check whether the field rules are empty
    // Usefule for optimization in validation checking
    pub fn is_none(&self) -> bool {
        !self.ignore
            && self.max_value.is_none()
            && self.min_value.is_none()
            && self.max_length.is_none()
            && self.min_length.is_none()
            && self.length.is_none()
            && self.regex.is_none()
            && !self.sign
    }

    pub fn set_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn set_min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    pub fn set_length(mut self, length: usize) -> Self {
        self.length = Some(length);
        self
    }

    pub fn set_ignore(mut self, ignore: bool) -> Self {
        self.ignore = ignore;
        self
    }

    #[cfg(feature = "regex")]
    pub fn set_regex(mut self, value: &str) -> Self {
        self.regex = Regex::from_str(value).ok();
        self
    }

    // Return the max length of the string
    pub fn max_length(&self) -> Option<usize> {
        self.max_length
    }

    // Return the min length of the string
    pub fn min_length(&self) -> Option<usize> {
        self.min_length
    }

    // Return the length of the string
    pub fn length(&self) -> Option<usize> {
        self.length
    }

    #[cfg(feature = "regex")]
    pub fn regex(&self) -> Option<&::regex::Regex> {
        self.regex.as_ref()
    }

    // Return if the field should be ignored
    pub fn ignore(&self) -> bool {
        self.ignore
    }

    // Return if the field should be signed
    pub fn sign(&self) -> bool {
        self.sign
    }

    #[cfg(feature = "regex")]
    pub fn check_regex_field_rules(&self, data: &str) -> Result<(), Error> {
        if let Some(regex_rules) = self.regex() {
            if !regex_rules.is_match(data) {
                let msg = format!("data does not match regex: {regex_rules}");
                return Err(Error::FieldRulesRegex(msg));
            }
        }

        Ok(())
    }

    pub fn check_data_length_field_rules(&self, length: usize) -> Result<(), Error> {
        if length > MAX_FIELD_SIZE {
            let msg = format!("data size exceeds 1 gigabyte");
            return Err(Error::FieldRulesLength(msg));
        }

        if let Some(length_rule) = self.length {
            if length != length_rule {
                let msg = format!("data size does not match required length: {length}");
                return Err(Error::FieldRulesLength(msg));
            }
        } else {
            // If exact length is not set, check the min and max length values.
            if let Some(max_length) = self.max_length {
                if length > max_length {
                    let msg = format!("data size exceeds field max length: {max_length}");
                    return Err(Error::FieldRulesLength(msg));
                }
            }

            if let Some(min_length) = self.min_length {
                if length < min_length {
                    let msg = format!("data size is less than min length: {min_length}");
                    return Err(Error::FieldRulesLength(msg));
                }
            };
        }

        Ok(())
    }

    pub fn check_numeric_value(&self, value: &NumericValue) -> Result<(), Error> {
        if let Some(max_value) = &self.max_value {
            if value > max_value {
                let msg = format!("data value exceeds field max value: {:?}", max_value);
                return Err(Error::FieldRulesValue(msg));
            }
        }

        if let Some(min_value) = &self.min_value {
            if value < min_value {
                let msg = format!("data value is less than min value: {:?}", min_value);
                return Err(Error::FieldRulesValue(msg));
            }
        };

        Ok(())
    }

    // pub fn le_bytes_data_length(length: usize) -> LeBytes {
    //     if length <= U8_MAX {
    //         LeBytes::U8((length as u8).to_le_bytes())
    //     } else if U8_MAX < length && length <= U16_MAX {
    //         LeBytes::U16((length as u16).to_le_bytes())
    //     } else if U16_MAX < length && length <= U32_MAX {
    //         LeBytes::U32((length as u32).to_le_bytes())
    //     } else if U32_MAX < length && length <= U64_MAX {
    //         LeBytes::U64((length as u64).to_le_bytes())
    //     } else {
    //         LeBytes::U32((length as u32).to_le_bytes())
    //     }
    // }

    // pub fn encoded_data_length(&self, bytes: &[u8]) -> usize {
    //     if let Some(length) = self.length {
    //         FieldRules::le_bytes_data_length(length).len()
    //     } else if let Some(max_length) = self.max_length {
    //         FieldRules::le_bytes_data_length(max_length).len()
    //     } else {
    //         if bytes.is_empty() {
    //             0
    //         } else if bytes.len() < DEFAULT_FIELD_LENGTH_LE_BYTES {
    //             1
    //         } else {
    //             DEFAULT_FIELD_LENGTH_LE_BYTES
    //             // // DEFAULT_FIELD_LENGTH_LE_BYTES
    //             // match bytes[0..DEFAULT_FIELD_LENGTH_LE_BYTES] {
    //             //     [0, 0, 0, _] => 4,
    //             //     [0, 0, _, _] => 3,
    //             //     [0, _, _, _] => 2,
    //             //     _ => 1,
    //             // }
    //         }
    //     }
    // }
}

#[derive(Debug, Clone)]
pub enum VTableFieldType<'a> {
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
    Vec,
    Bytes,
    Bool,
    Struct(StructName<'a>),
    HashMap {
        key: Box<VTableFieldType<'a>>,
        value: Box<VTableFieldType<'a>>,
    },
}

impl<'a> std::fmt::Display for VTableFieldType<'a> {
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
            VTableFieldType::Vec => write!(f, "Vec"),
            VTableFieldType::Bytes => write!(f, "Bytes"),
            VTableFieldType::Bool => write!(f, "bool"),
            VTableFieldType::Struct(s) => write!(f, "{}", s),
            VTableFieldType::HashMap { key, value } => {
                write!(f, "HashMap<{}, {}>", key, value)
            }
        }
    }
}

impl<'a> VTableFieldType<'a> {
    pub fn is_struct(field_type: impl TryInto<Self>) -> bool {
        // println!("Checking if field type is a struct");

        match field_type.try_into() {
            Ok(VTableFieldType::Struct(_)) => {
                // println!("Field Type is a struct: {:?}", s);
                true
            }
            _ => false,
        }
    }

    pub fn hashmap_from_str(input: &str) -> VTableFieldType {
        let mut key_value = input.split('<').collect::<Vec<&str>>();
        key_value[1] = key_value[1].trim_end_matches('>');
        let key = key_value[1].split(',').collect::<Vec<&str>>()[0].trim();
        let value = key_value[1].split(',').collect::<Vec<&str>>()[1].trim();

        // println!("Key: {:?}", key);
        // println!("Value: {:?}", value);

        VTableFieldType::HashMap {
            key: Box::new(VTableFieldType::from(key)),
            value: Box::new(VTableFieldType::from(value)),
        }
    }
}

impl<'a> From<&'a str> for VTableFieldType<'a> {
    fn from(s: &'a str) -> Self {
        // println!("Converting string to field type: {:?}", s);

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
            s if s.contains("str") => VTableFieldType::Str,
            // "& 'static str" => VTableFieldType::Str,
            // "& 'a str" => VTableFieldType::Str,
            // "&str" => VTableFieldType::Str,
            "Vec < u8 >" => VTableFieldType::Bytes,
            s if s.contains("[u8]") => VTableFieldType::Bytes,
            s if s.contains("[u8 ; ") => VTableFieldType::Bytes,
            // "&[u8]" => VTableFieldType::Bytes,
            // "[u8]" => VTableFieldType::Bytes,
            s if s.contains("Vec") => VTableFieldType::Vec,
            "bool" => VTableFieldType::Bool,
            s if s.contains("HashMap") => VTableFieldType::hashmap_from_str(s),
            s => VTableFieldType::Struct(s),
        }
    }
}

impl<'a> From<u8> for VTableFieldType<'a> {
    fn from(byte: u8) -> Self {
        match byte {
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
            16 => VTableFieldType::Vec,
            17 => VTableFieldType::Bytes,
            18 => VTableFieldType::Bool,
            19 => VTableFieldType::Struct(""),
            _ => todo!("Handle unknown field type"),
        }
    }
}

impl<'a> Into<u8> for VTableFieldType<'a> {
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
            VTableFieldType::Vec => 16,
            VTableFieldType::Bytes => 17,
            VTableFieldType::Bool => 18,
            VTableFieldType::Struct(_) => 19,
            VTableFieldType::HashMap { key: _, value: _ } => 20, //
        }
    }
}

// Implement DocBufDecodeField for VTableField

impl<'a> DocBufDecodeField<String> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<String, Error> {
        match self.field_type {
            VTableFieldType::String | VTableFieldType::HashMap { .. } => {
                let length =
                    u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

                // Consume the data from the buffer
                buffer.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

                let data = String::from_utf8(buffer.drain(0..length).collect())?;

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

// impl<'a> DocBufDecodeField<&'a str> for VTableField<'a> {
//     fn decode(&self, buffer: &mut Vec<u8>) -> Result<&'a str, Error> {
//         match self.field_type {
//             VTableFieldType::Str => {
//                 let length =
//                     u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

//                 let data = std::str::from_utf8(
//                     &buffer[DEFAULT_FIELD_LENGTH_LE_BYTES..DEFAULT_FIELD_LENGTH_LE_BYTES + length],
//                 )?;

//                 // Consume the data from the buffer
//                 buffer.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES + length);

//                 Ok(data)
//             }
//             _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
//         }
//     }
// }

impl<'a> DocBufDecodeField<bool> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<bool, Error> {
        match self.field_type {
            VTableFieldType::Bool => {
                let data = buffer[0] == 1;

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u8> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u8, Error> {
        match self.field_type {
            VTableFieldType::U8 => {
                let data = buffer[0];

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u16> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u16, Error> {
        match self.field_type {
            VTableFieldType::U16 => {
                let data = u16::from_le_bytes([buffer[0], buffer[1]]);

                // Consume the data from the buffer
                buffer.drain(0..2);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u32> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u32, Error> {
        match self.field_type {
            VTableFieldType::U32 => {
                let data = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u64> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u64, Error> {
        match self.field_type {
            VTableFieldType::U64 | VTableFieldType::USIZE => {
                let data = u64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u128> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u128, Error> {
        match self.field_type {
            VTableFieldType::U128 => {
                let data = u128::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
                    buffer[13], buffer[14], buffer[15],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..16);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<usize> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<usize, Error> {
        match self.field_type {
            VTableFieldType::USIZE => {
                let data = usize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i8> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i8, Error> {
        match self.field_type {
            VTableFieldType::I8 => {
                let data = i8::from_le_bytes([buffer[0]]);

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i16> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i16, Error> {
        match self.field_type {
            VTableFieldType::I16 => {
                let data = i16::from_le_bytes([buffer[0], buffer[1]]);

                // Consume the data from the buffer
                buffer.drain(0..2);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i32> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i32, Error> {
        match self.field_type {
            VTableFieldType::I32 => {
                let data = i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i64> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i64, Error> {
        match self.field_type {
            VTableFieldType::I64 | VTableFieldType::ISIZE => {
                let data = i64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i128> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i128, Error> {
        match self.field_type {
            VTableFieldType::I128 => {
                let data = i128::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
                    buffer[13], buffer[14], buffer[15],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..16);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<isize> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<isize, Error> {
        match self.field_type {
            VTableFieldType::ISIZE => {
                let data = isize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<f32> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<f32, Error> {
        match self.field_type {
            VTableFieldType::F32 => {
                let data = f32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<f64> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<f64, Error> {
        match self.field_type {
            VTableFieldType::F64 => {
                let data = f64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<Vec<u8>> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        match self.field_type {
            VTableFieldType::Bytes => {
                let length =
                    u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

                // Consume the data from the buffer
                buffer.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

                Ok(buffer.drain(0..length).collect())
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}
