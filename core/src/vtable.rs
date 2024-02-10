mod fields;
mod structs;
mod table;

pub use fields::*;
pub use structs::*;
pub use table::*;

/// The separator value type of `0xFF` is used to separate the structs in the VTable
pub const STRUCT_SEPARATOR: u8 = 0xFF;

/// The separator value type of `0xFE` is used to separate the fields in the VTable
pub const FIELD_SEPARATOR: u8 = 0xFE;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Field length mismatch")]
    FieldLengthMismatch,
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