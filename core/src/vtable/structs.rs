use std::collections::HashMap;

use super::*;

pub type StructIndex = u8;
pub type StructNameAsBytes = Vec<u8>;

#[derive(Debug, Clone)]
pub struct VTableStruct {
    pub struct_index: StructIndex,
    pub struct_name_as_bytes: Vec<u8>,
    pub fields: HashMap<FieldNameAsBytes, VTableField>,
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

        let field = VTableField::new(
            self.struct_index,
            field_type.into(),
            field_index,
            field_name,
        );
        self.fields.insert(field.clone().field_name_as_bytes, field);
        self.num_fields += 1;
    }

    pub fn set_struct_index(&mut self, index: StructIndex) {
        self.struct_index = index;

        for field in self.fields.values_mut() {
            field.struct_index = self.struct_index;
        }
    }

    // Return the field index from the struct
    pub fn field_index_from_name(&self, name: &str) -> Result<FieldIndex, Error> {
        for field in self.fields.values() {
            if field.field_name_as_bytes == name.as_bytes() {
                return Ok(field.field_index);
            }
        }

        Err(Error::FieldNotFound)
    }

    // Return the field by name from the struct
    pub fn field_by_name(&self, name: impl AsRef<[u8]>) -> Result<&VTableField, Error> {
        for field in self.fields.values() {
            if field.field_name_as_bytes == name.as_ref() {
                return Ok(field);
            }
        }

        Err(Error::FieldNotFound)
    }

    // Return the field by index from the struct
    pub fn field_by_index(&self, index: &FieldIndex) -> Result<&VTableField, Error> {
        for field in self.fields.values() {
            if field.field_index == *index {
                return Ok(field);
            }
        }

        Err(Error::FieldNotFound)
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
