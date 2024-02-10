use std::collections::HashMap;

use super::*;

pub type StructIndex = u8;
pub type StructNameAsBytes = Vec<u8>;

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