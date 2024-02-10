use std::collections::HashMap;

/// The separator value type of `0xFF` is used to separate the structs in the VTable
pub const STRUCT_SEPARATOR: u8 = 0xFF;

/// The separator value type of `0xFE` is used to separate the fields in the VTable
pub const FIELD_SEPARATOR: u8 = 0xFE;


#[derive(Debug, Clone)]
pub struct VTable {
    pub structs: HashMap<StructNameAsBytes, VTableStruct>,
    pub num_structs: StructIndex,
}

impl VTable {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            num_structs: 0,
        }
    }

    pub fn add_struct(&mut self, vtable_struct: VTableStruct) {
        let mut vtable_struct = vtable_struct;
        vtable_struct.set_struct_index(self.num_structs);
        self.structs.insert(vtable_struct.struct_name_as_bytes.clone(), vtable_struct.clone());
        self.num_structs += 1;

        
    }

    pub fn merge_vtable(&mut self, vtable: VTable) {
        for vtable_struct in vtable.structs.values() {
            self.add_struct(vtable_struct.clone());
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for vtable_struct in self.structs.values() {
            let packed_bytes = vtable_struct.to_bytes();
            println!("VTable Struct: {:?}", packed_bytes);
            bytes.extend_from_slice(&packed_bytes);
            // Add a separator value type of `0xFF`
            bytes.push(STRUCT_SEPARATOR);
        }

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

pub type StructIndex = u8;
pub type StructNameAsBytes = Vec<u8>;
pub type FieldIndex = u8;

#[derive(Debug, Clone)]
pub struct VTableStruct {
    pub struct_index: StructIndex,
    pub struct_name_as_bytes: Vec<u8>,
    pub fields: HashMap<FieldIndex, VTableField>,
    pub num_fields: FieldIndex,
}

impl VTableStruct {
    pub fn new(struct_name: &str, index: Option<u8>) -> Self {
        Self {
            struct_index: index.unwrap_or_default(),
            struct_name_as_bytes: struct_name.as_bytes().to_vec(),
            fields: HashMap::new(),
            num_fields: 0,
        }
    }

    pub fn add_field(&mut self, field_type: impl Into<FieldType>, field_name: &str) {
        let field_index = self.num_fields;

        let field = VTableField::new(self.struct_index, field_type.into(), field_index, field_name);
        self.fields.insert(field_index, field);
        self.num_fields += 1;
    }

    pub fn set_struct_index(&mut self, index: StructIndex) {
        self.struct_index = index;

        for field in self.fields.values_mut() {
            field.struct_index = self.struct_index;
        }
    }


    // Pack the VTableStruct into a byte array
    // [struct_index,struct_name_len,struct_name_bytes,num_fields,]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.struct_index);
        // Add the number of fields
        bytes.push(self.num_fields);
        // struct name length
        bytes.push(self.struct_name_as_bytes.len() as u8);
        // struct name in bytes
        bytes.extend_from_slice(self.struct_name_as_bytes.as_slice());
        
        // Add the fields
        for field in self.fields.values() {
            println!("VTable Field: {:?}", field.to_bytes());
            bytes.extend_from_slice(&field.to_bytes());
            bytes.push(FIELD_SEPARATOR);
        }


        bytes
    }

}

#[derive(Debug, Clone)]
pub struct VTableField {
    // The index of the struct this field belongs to
    struct_index: StructIndex,
    // The type of the field
    field_type: FieldType,
    field_index: FieldIndex,
    field_name_as_bytes: Vec<u8>,
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
    fn to_bytes(&self) -> Vec<u8> {
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::error;

    #[derive(Default)]
    struct ExampleDataIntegers {
        pub field_i8: i8,
        pub field_i16: i16,
        pub field_i32: i32,
        pub field_i64: i64,
        pub field_i128: i128,
    }

    #[derive(Default)]
    struct ExampleDataFloatingPoint {
        pub field_f32: f32,
        pub field_f64: f64,
    }

    impl ExampleDataFloatingPoint {
        pub fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            
            let field_f32 = self.field_f32.to_be_bytes();
            bytes.push(field_f32.len() as u8);
            bytes.extend_from_slice(&field_f32);
            // Add field separator
            bytes.push(FIELD_SEPARATOR);


            let field_f64 = self.field_f64.to_be_bytes();
            bytes.push(field_f64.len() as u8);
            bytes.extend_from_slice(&field_f64);
            // Add field separator
            bytes.push(FIELD_SEPARATOR);

            // Lastly, add the struct separator value
            bytes.push(STRUCT_SEPARATOR);
            bytes
        }
    }

    #[derive(Default)]
    struct ExampleUnsignedIntegers {
        pub field_u8: u8,
        pub field_u16: u16,
        pub field_u32: u32,
        pub field_u64: u64,
        pub field_u128: u128,
        pub field_usize: usize,
    }

    #[derive(Default)]
    struct ExampleData {
        pub field_one: &'static [u8],
        pub field_two: &'static str,
        pub field_three: String,
        pub field_four: Vec<u8>,
        pub field_five: bool,
        pub field_six: ExampleDataFloatingPoint,
        pub integers: ExampleDataIntegers,
        pub unsigned_integers: ExampleUnsignedIntegers,
    }

    impl ExampleData {
        fn new() -> Self {
            Self {
                field_one: b"field_one",
                field_two: "field_two",
                field_three: "field_three".to_string(),
                field_four: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
                field_five: true,
                field_six: ExampleDataFloatingPoint {
                    field_f32: 3.14,
                    field_f64: 3.14159,
                },
                integers: ExampleDataIntegers {
                    field_i8: i8::MAX,
                    field_i16: i16::MAX,
                    field_i32: i32::MAX,
                    field_i64: i64::MAX,
                    field_i128: i128::MAX,
                },
                unsigned_integers: ExampleUnsignedIntegers {
                    field_u8: u8::MAX,
                    field_u16: u16::MAX,
                    field_u32: u32::MAX,
                    field_u64: u64::MAX,
                    field_u128: u128::MAX,
                    field_usize: usize::MAX,
                },
            }
        }

        pub fn struct_name() -> &'static str {
            "ExampleData"
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            let mut bytes = Vec::new();

            // Add field one to the bytes
            bytes.push(self.field_one.len() as u8);
            bytes.extend_from_slice(self.field_one);
            // Add field separator value
            bytes.push(FIELD_SEPARATOR);

            // Add field two to the bytes
            bytes.push(self.field_two.len() as u8);
            bytes.extend_from_slice(self.field_two.as_bytes());
            // Add field separator value
            bytes.push(FIELD_SEPARATOR);

            // Add field three to the bytes
            bytes.push(self.field_three.len() as u8);
            bytes.extend_from_slice(self.field_three.as_bytes());
            // Add field separator value
            bytes.push(FIELD_SEPARATOR);

            // Add field four to the bytes
            bytes.push(self.field_four.len() as u8);
            bytes.extend_from_slice(&self.field_four);
            // Add field separator value
            bytes.push(FIELD_SEPARATOR);

            // Add field five to the bytes
            bytes.push(1u8); // bool has a fixed size of 1 byte
            bytes.push(self.field_five as u8);
            // Add field separator value
            bytes.push(FIELD_SEPARATOR);

            // Add field six to the bytes
            bytes.extend_from_slice(&self.field_six.to_bytes());


            // Lastly, add the struct separator value
            bytes.push(STRUCT_SEPARATOR);

            bytes

        }

        pub fn vtable() -> VTable {
            let mut vtable = VTable::new();
        
            // Create the struct
            let mut vtable_struct = VTableStruct::new(ExampleData::struct_name(), None);
            
            // Add Fields
            vtable_struct.add_field(FieldType::Bytes, "field_one");
            vtable_struct.add_field(FieldType::String, "field_two");
            vtable_struct.add_field(FieldType::String, "field_three");
            vtable_struct.add_field(FieldType::Vec, "field_four");
            vtable_struct.add_field(FieldType::Bool, "field_five");
            
            // Add struct information to the vtable
            vtable_struct.add_field(FieldType::Struct(b"ExampleDataFloatingPoint".to_vec()), "field_six");
            
            // vtable_struct.add_field(FieldType::F32, "field_six");
            // vtable_struct.add_field(FieldType::F64, "field_seven");
            // vtable_struct.add_field(FieldType::I8, "field_eight");
            // vtable_struct.add_field(FieldType::I16, "field_nine");
            // vtable_struct.add_field(FieldType::I32, "field_ten");
            // vtable_struct.add_field(FieldType::I64, "field_eleven");
            // vtable_struct.add_field(FieldType::I128, "field_twelve");
            // vtable_struct.add_field(FieldType::U8, "field_thirteen");
            // vtable_struct.add_field(FieldType::U16, "field_fourteen");
            // vtable_struct.add_field(FieldType::U32, "field_fifteen");
            // vtable_struct.add_field(FieldType::U64, "field_sixteen");

            // Add the struct the vtable
            vtable.add_struct(vtable_struct);

            let mut field_six = VTableStruct::new("ExampleDataFloatingPoint", Some(1));
            field_six.add_field(FieldType::F32, "field_f32");
            field_six.add_field(FieldType::F64, "field_f64");


            // Add the struct the vtable
            vtable.add_struct(field_six);

            vtable
        }

        pub fn from_vtable_bytes(bytes: &[u8]) -> Result<Self, error::Error> {
            let vtable = ExampleData::vtable();
            let mut data = bytes.to_vec();

            // let mut map = HashMap::new();

            // Parse the number of field data
            let mut index = 0;
            let field_length = data[index];

    
            Ok(Self {
                ..Default::default()
            })
        }
    }

    #[test]
    fn test_vtable_pack_unpack() {
        let data = ExampleData::new();
        
        let vtable = ExampleData::vtable();

        let bytes = vtable.to_bytes();

        println!("\n\nVTable Bytes: {:?}", bytes);

        let file = data.to_bytes();

        println!("\n\nFile Bytes: {:?}", file);
        
        let data2 = ExampleData::from_vtable_bytes(&file);
        
    }
}