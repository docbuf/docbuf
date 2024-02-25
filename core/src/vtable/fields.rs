use std::{cmp::Ordering, str::FromStr};

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LeBytes {
    U8([u8; 1]),
    U16([u8; 2]),
    U32([u8; 4]),
    U64([u8; 8]),
}

impl LeBytes {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            LeBytes::U8(bytes) => bytes,
            LeBytes::U16(bytes) => bytes,
            LeBytes::U32(bytes) => bytes,
            LeBytes::U64(bytes) => bytes,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LeBytes::U8(_) => 1,
            LeBytes::U16(_) => 2,
            LeBytes::U32(_) => 4,
            LeBytes::U64(_) => 8,
        }
    }
}

pub type FieldIndex = u8;
// pub type FieldNameAsBytes<'a> = &'a [u8];
pub type FieldName<'a> = &'a str;

#[derive(Debug, Clone)]
pub struct VTableField<'a> {
    // The index of the vtable item this field belongs to
    pub item_index: VTableItemIndex,
    // The type of the field
    pub field_type: FieldType<'a>,
    pub field_index: FieldIndex,
    pub field_name: FieldName<'a>,
    pub field_rules: FieldRules,
}

impl<'a> VTableField<'a> {
    pub fn new(
        item_index: VTableItemIndex,
        field_type: FieldType<'a>,
        field_index: FieldIndex,
        field_name: FieldName<'a>,
        field_rules: FieldRules,
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
            FieldType::String => {
                // prepend length to the field data
                let length = (field_data.len() as u32).to_le_bytes();

                output.extend_from_slice(length.as_slice());
            }
            FieldType::HashMap { key, value } => {
                // prepend length to the field data
                let length = (field_data.len() as u32).to_le_bytes();

                output.extend_from_slice(length.as_slice());
            }
            _ => {}
        };

        output.extend_from_slice(field_data.as_bytes());

        // println!("encode_str OUTPUT: {:?}", output);

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

    pub fn decode(&self, bytes: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(match &self.field_type {
            FieldType::String => self.decode_string_data(bytes)?,
            FieldType::U8 | FieldType::I8 => {
                let field_data = bytes.drain(0..1);

                field_data.collect()
            }
            FieldType::U16 | FieldType::I16 => {
                let field_data = bytes.drain(0..2);

                field_data.collect()
            }
            FieldType::U32 | FieldType::F32 | FieldType::I32 => {
                let field_data = bytes.drain(0..4);

                field_data.collect()
            }
            FieldType::U64
            | FieldType::F64
            | FieldType::USIZE
            | FieldType::I64
            | FieldType::ISIZE => {
                let field_data = bytes.drain(0..8);

                field_data.collect()
            }
            FieldType::U128 | FieldType::I128 => {
                let field_data = bytes.drain(0..16);

                field_data.collect()
            }

            FieldType::Struct(_) => Vec::new(),
            FieldType::HashMap { key, value } => self.decode_map_data(key, value, bytes)?,
            _ => {
                unimplemented!("Decode Field Type: {:#?}", self.field_type);
            }
        })
    }

    pub fn decode_map_data(
        &self,
        key: &Box<FieldType>,
        value: &Box<FieldType>,
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
            let field_data_length = self.decode_field_data_length(bytes);

            // Remove the encoded length from the bytes
            // bytes.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

            // Remove the field data from the bytes and return it
            output.extend_from_slice(
                bytes
                    .drain(0..field_data_length + DEFAULT_FIELD_LENGTH_LE_BYTES)
                    .as_slice(),
            );

            // println!("Remaing Bytes: {:?}", bytes);
        }

        // println!("Output: {:?}", output);

        Ok(output)
    }

    // Decodes the field data from the bytes input.
    // Returns the raw field data and removes the field data from the bytes, including its length.
    pub fn decode_string_data(&self, bytes: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        let field_data_length = self.decode_field_data_length(bytes);

        // Remove the encoded length from the bytes
        bytes.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);
        // Remove the field data from the bytes and return it
        let field_data = bytes.drain(0..field_data_length);

        Ok(field_data.collect())
    }

    pub fn decode_field_data_length(&self, bytes: &[u8]) -> usize {
        let mut field_length = [0u8; DEFAULT_FIELD_LENGTH_LE_BYTES];

        for byte in 0..DEFAULT_FIELD_LENGTH_LE_BYTES {
            field_length[byte] = bytes[byte];
        }

        u32::from_le_bytes(field_length) as usize
    }

    pub fn validate_str(&self, data: &str) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        match self.field_type {
            FieldType::String => {
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
pub struct FieldRules {
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

impl FieldRules {
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
pub enum FieldType<'a> {
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
        key: Box<FieldType<'a>>,
        value: Box<FieldType<'a>>,
    },
}

impl<'a> FieldType<'a> {
    pub fn is_struct(field_type: impl TryInto<Self>) -> bool {
        // println!("Checking if field type is a struct");

        match field_type.try_into() {
            Ok(FieldType::Struct(_)) => {
                // println!("Field Type is a struct: {:?}", s);
                true
            }
            _ => false,
        }
    }

    pub fn hashmap_from_str(input: &str) -> FieldType {
        let mut key_value = input.split('<').collect::<Vec<&str>>();
        key_value[1] = key_value[1].trim_end_matches('>');
        let key = key_value[1].split(',').collect::<Vec<&str>>()[0].trim();
        let value = key_value[1].split(',').collect::<Vec<&str>>()[1].trim();

        // println!("Key: {:?}", key);
        // println!("Value: {:?}", value);

        FieldType::HashMap {
            key: Box::new(FieldType::from(key)),
            value: Box::new(FieldType::from(value)),
        }
    }
}

impl<'a> From<&'a str> for FieldType<'a> {
    fn from(s: &'a str) -> Self {
        // println!("Converting string to field type: {:?}", s);

        match s {
            "u8" => FieldType::U8,
            "u16" => FieldType::U16,
            "u32" => FieldType::U32,
            "u64" => FieldType::U64,
            "u128" => FieldType::U128,
            "usize" => FieldType::USIZE,
            "i8" => FieldType::I8,
            "i16" => FieldType::I16,
            "i32" => FieldType::I32,
            "i64" => FieldType::I64,
            "i128" => FieldType::I128,
            "isize" => FieldType::ISIZE,
            "f32" => FieldType::F32,
            "f64" => FieldType::F64,
            "String" => FieldType::String,
            s if s.contains("str") => FieldType::Str,
            // "& 'static str" => FieldType::Str,
            // "& 'a str" => FieldType::Str,
            // "&str" => FieldType::Str,
            "Vec<u8>" => FieldType::Bytes,
            s if s.contains("[u8]") => FieldType::Bytes,
            // "&[u8]" => FieldType::Bytes,
            // "[u8]" => FieldType::Bytes,
            "Vec" => FieldType::Vec,
            "bool" => FieldType::Bool,
            s if s.contains("HashMap") => FieldType::hashmap_from_str(s),
            s => FieldType::Struct(s),
        }
    }
}

impl<'a> From<u8> for FieldType<'a> {
    fn from(byte: u8) -> Self {
        match byte {
            0 => FieldType::U8,
            1 => FieldType::U16,
            2 => FieldType::U32,
            3 => FieldType::U64,
            4 => FieldType::U128,
            5 => FieldType::USIZE,
            6 => FieldType::I8,
            7 => FieldType::I16,
            8 => FieldType::I32,
            9 => FieldType::I64,
            10 => FieldType::I128,
            11 => FieldType::ISIZE,
            12 => FieldType::F32,
            13 => FieldType::F64,
            14 => FieldType::String,
            15 => FieldType::Str,
            16 => FieldType::Vec,
            17 => FieldType::Bytes,
            18 => FieldType::Bool,
            19 => FieldType::Struct(""),
            _ => todo!("Handle unknown field type"),
        }
    }
}

impl<'a> Into<u8> for FieldType<'a> {
    fn into(self) -> u8 {
        match self {
            FieldType::U8 => 0,
            FieldType::U16 => 1,
            FieldType::U32 => 2,
            FieldType::U64 => 3,
            FieldType::U128 => 4,
            FieldType::USIZE => 5,
            FieldType::I8 => 6,
            FieldType::I16 => 7,
            FieldType::I32 => 8,
            FieldType::I64 => 9,
            FieldType::I128 => 10,
            FieldType::ISIZE => 11,
            FieldType::F32 => 12,
            FieldType::F64 => 13,
            FieldType::String => 14,
            FieldType::Str => 15,
            FieldType::Vec => 16,
            FieldType::Bytes => 17,
            FieldType::Bool => 18,
            FieldType::Struct(_) => 19,
            FieldType::HashMap { key: _, value: _ } => 20, //
        }
    }
}
