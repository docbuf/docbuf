pub use super::*;

// Number of bytes in a gigabyte as a usize
pub const GIGABYTES: usize = 1024 * 1024 * 1024;

// Maximum size of a field in a struct
pub const MAX_FIELD_SIZE: usize = GIGABYTES;

// Default field length encoded as 4 le bytes
pub const DEFAULT_FIELD_LENGTH_LE_BYTES: usize = 4;

pub const U8_MAX: usize = u8::MAX as usize;
pub const U16_MAX: usize = u16::MAX as usize;
pub const U32_MAX: usize = u32::MAX as usize;
pub const U64_MAX: usize = u64::MAX as usize;

pub type FieldIndex = u8;
pub type FieldNameAsBytes = Vec<u8>;

#[derive(Debug, Clone)]
pub struct VTableField {
    // The index of the struct this field belongs to
    pub struct_index: StructIndex,
    // The type of the field
    pub field_type: FieldType,
    pub field_index: FieldIndex,
    pub field_name_as_bytes: FieldNameAsBytes,
    pub field_rules: FieldRules,
}

impl VTableField {
    pub fn new(
        struct_index: StructIndex,
        field_type: FieldType,
        field_index: FieldIndex,
        field_name: &str,
        field_rules: FieldRules,
    ) -> Self {
        // println!("Field Rules: {:?}", field_rules);

        Self {
            struct_index,
            field_type,
            field_index,
            field_name_as_bytes: field_name.as_bytes().to_vec(),
            field_rules,
        }
    }

    pub fn encode_field_data(&self, field_data: &[u8]) -> Result<Vec<u8>, Error> {
        let data_length = self.encode_data_length_to_le_bytes(field_data.len())?;

        let mut encoded = Vec::with_capacity(field_data.len() + data_length.len());
        // Add the length of the data
        encoded.extend_from_slice(data_length.as_slice());
        // Add the field data
        encoded.extend_from_slice(&field_data);

        Ok(encoded)
    }

    pub fn encode_data_length_to_le_bytes(
        &self,
        field_data_length: usize,
    ) -> Result<Vec<u8>, Error> {
        self.field_rules
            .check_data_length_field_rules(field_data_length)?;

        let le_bytes = FieldRules::le_bytes_data_length(field_data_length);

        Ok(le_bytes)
    }

    pub fn decode(&self, bytes: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(match &self.field_type {
            FieldType::String => self.decode_field_data(bytes)?,
            FieldType::U8 => {
                let field_data = bytes.drain(0..1);

                field_data.collect()
            }
            FieldType::Struct(_) => Vec::new(),
            _ => {
                unimplemented!("parse_bytes_input Field Type: {:#?}", self.field_type);
            }
        })
    }

    // Decodes the field data from the bytes input.
    // Returns the raw field data and removes the field data from the bytes, including its length.
    pub fn decode_field_data(&self, bytes: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        let (encoded_length, field_data_length) = self.decode_field_data_length(bytes);

        if encoded_length == 0 {
            return Ok(Vec::new());
        }

        // Remove the encoded length from the bytes
        bytes.drain(0..encoded_length);
        // Remove the field data from the bytes and return it
        let field_data = bytes.drain(0..field_data_length);

        Ok(field_data.collect())
    }

    pub fn decode_field_data_length(&self, bytes: &[u8]) -> (usize, usize) {
        let mut field_length = [0u8; DEFAULT_FIELD_LENGTH_LE_BYTES];
        let encoded_length = self.field_rules.encoded_data_length(bytes);

        for byte in 0..encoded_length {
            field_length[byte] = bytes[byte];
        }

        (encoded_length, u32::from_le_bytes(field_length) as usize)
    }

    /// Pack the VTableField into a byte array
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // struct index field belongs to
        bytes.push(self.struct_index);

        // field index
        bytes.push(self.field_index);

        // field type
        bytes.push(self.field_type.clone().into());

        // field name length
        bytes.push(self.field_name_as_bytes.len() as u8);
        bytes.extend_from_slice(self.field_name_as_bytes.as_slice());
        bytes
    }

    pub fn field_name_as_string(&self) -> Result<String, Error> {
        let name = String::from_utf8(self.field_name_as_bytes.clone())?;
        Ok(name)
    }
}

/// Optional rules for a field
#[derive(Debug, Clone)]
pub struct FieldRules {
    pub ignore: bool,
    pub max_value: Option<usize>,
    pub min_value: Option<usize>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    // An absolute length
    pub length: Option<usize>,
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
            sign: false,
        }
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

    // Return if the field should be ignored
    pub fn ignore(&self) -> bool {
        self.ignore
    }

    // Return if the field should be signed
    pub fn sign(&self) -> bool {
        self.sign
    }

    pub fn check_data_length_field_rules(&self, length: usize) -> Result<(), Error> {
        if length > MAX_FIELD_SIZE {
            let msg = format!("data size exceeds 1 gigabyte");
            return Err(Error::FieldRulesLength(msg));
        } else if let Some(length) = self.length {
            if length != length as usize {
                let msg = format!("data size does not match required length: {length}");
                return Err(Error::FieldRulesLength(msg));
            }
        } else if let Some(max_length) = self.max_length {
            if length > max_length as usize {
                let msg = format!("data size exceeds field max length: {max_length}");
                return Err(Error::FieldRulesLength(msg));
            }
        } else if let Some(min_length) = self.min_length {
            if length < min_length as usize {
                let msg = format!("data size is less than min length: {min_length}");
                return Err(Error::FieldRulesLength(msg));
            }
        };

        Ok(())
    }

    pub fn le_bytes_data_length(length: usize) -> Vec<u8> {
        if length <= U8_MAX {
            (length as u8).to_le_bytes().to_vec()
        } else if U8_MAX < length && length <= U16_MAX {
            (length as u16).to_le_bytes().to_vec()
        } else if U16_MAX < length && length <= U32_MAX {
            (length as u32).to_le_bytes().to_vec()
        } else if U32_MAX < length && length <= U64_MAX {
            (length as u64).to_le_bytes().to_vec()
        } else {
            (length as u32).to_le_bytes().to_vec()
        }
    }

    pub fn encoded_data_length(&self, bytes: &[u8]) -> usize {
        if let Some(length) = self.length {
            return FieldRules::le_bytes_data_length(length).len();
        } else if let Some(max_length) = self.max_length {
            return FieldRules::le_bytes_data_length(max_length).len();
        } else {
            if 0 < bytes.len() && bytes.len() >= DEFAULT_FIELD_LENGTH_LE_BYTES {
                match [bytes[0], bytes[1], bytes[2], bytes[3]] {
                    [0, 0, 0, _] => 4,
                    [0, 0, _, _] => 3,
                    [0, _, _, _] => 2,
                    _ => 1,
                }
            } else {
                return 0;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    String,
    Str,
    Vec,
    Bytes,
    Bool,
    Struct(StructNameAsBytes),
}

impl FieldType {
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
}

impl From<&str> for FieldType {
    fn from(s: &str) -> Self {
        // println!("Converting string to field type: {:?}", s);

        match s {
            "u8" => FieldType::U8,
            "u16" => FieldType::U16,
            "u32" => FieldType::U32,
            "u64" => FieldType::U64,
            "i8" => FieldType::I8,
            "i16" => FieldType::I16,
            "i32" => FieldType::I32,
            "i64" => FieldType::I64,
            "i128" => FieldType::I128,
            "f32" => FieldType::F32,
            "f64" => FieldType::F64,
            "String" => FieldType::String,
            "& 'static str" => FieldType::Str,
            "& 'a str" => FieldType::Str,
            "&str" => FieldType::Str,
            "Vec<u8>" => FieldType::Bytes,
            "&[u8]" => FieldType::Bytes,
            "[u8]" => FieldType::Bytes,
            "Vec" => FieldType::Vec,
            "bool" => FieldType::Bool,
            s => FieldType::Struct(s.as_bytes().to_vec()),
        }
    }
}

impl From<u8> for FieldType {
    fn from(byte: u8) -> Self {
        match byte {
            0 => FieldType::U8,
            1 => FieldType::U16,
            2 => FieldType::U32,
            3 => FieldType::U64,
            4 => FieldType::I8,
            5 => FieldType::I16,
            6 => FieldType::I32,
            7 => FieldType::I64,
            8 => FieldType::I128,
            9 => FieldType::F32,
            10 => FieldType::F64,
            11 => FieldType::String,
            12 => FieldType::Str,
            13 => FieldType::Vec,
            14 => FieldType::Bytes,
            15 => FieldType::Bool,
            16 => FieldType::Struct(StructNameAsBytes::new()),
            _ => todo!("Handle unknown field type"),
        }
    }
}

impl Into<u8> for FieldType {
    fn into(self) -> u8 {
        match self {
            FieldType::U8 => 0,
            FieldType::U16 => 1,
            FieldType::U32 => 2,
            FieldType::U64 => 3,
            FieldType::I8 => 4,
            FieldType::I16 => 5,
            FieldType::I32 => 6,
            FieldType::I64 => 7,
            FieldType::I128 => 8,
            FieldType::F32 => 9,
            FieldType::F64 => 10,
            FieldType::String => 11,
            FieldType::Str => 12,
            FieldType::Vec => 13,
            FieldType::Bytes => 14,
            FieldType::Bool => 15,
            FieldType::Struct(_) => 16,
        }
    }
}
