pub use super::structs::*;

pub type FieldIndex = u8;

#[derive(Debug, Clone)]
pub struct VTableField {
    // The index of the struct this field belongs to
    pub struct_index: StructIndex,
    // The type of the field
    pub field_type: FieldType,
    pub field_index: FieldIndex,
    pub field_name_as_bytes: Vec<u8>,
}

impl VTableField {
    pub fn new(
        struct_index: StructIndex,
        field_type: FieldType,
        field_index: FieldIndex,
        field_name: &str,
    ) -> Self {
        Self {
            struct_index,
            field_type,
            field_index,
            field_name_as_bytes: field_name.as_bytes().to_vec(),
        }
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
    Vec,
    Bytes,
    Bool,
    Struct(StructNameAsBytes),
}

impl FieldType {
    pub fn is_struct(field_type: impl TryInto<Self>) -> bool {
        println!("Checking if field type is a struct");

        match field_type.try_into() {
            Ok(FieldType::Struct(s)) => {
                println!("Field Type is a struct: {:?}", s);
                true
            },
            _ => false,
        }
    }
}

impl From<&str> for FieldType {
    fn from(s: &str) -> Self {
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
            12 => FieldType::Vec,
            13 => FieldType::Bytes,
            14 => FieldType::Bool,
            15 => FieldType::Struct(StructNameAsBytes::new()),
            _ => todo!("Handle unknown field type")
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
            FieldType::Vec => 12,
            FieldType::Bytes => 13,
            FieldType::Bool => 14,
            FieldType::Struct(_) => 15,
        }
    }
}

