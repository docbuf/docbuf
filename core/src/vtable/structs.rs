use std::collections::HashMap;

use super::*;

pub type StructIndex = u8;
pub type StructNameAsBytes<'a> = &'a [u8];

#[derive(Debug, Clone)]
pub struct VTableStruct<'a> {
    pub struct_index: StructIndex,
    pub struct_name_as_bytes: StructNameAsBytes<'a>,
    pub fields: HashMap<FieldNameAsBytes<'a>, VTableField<'a>>,
    pub num_fields: FieldIndex,
}

impl<'a> VTableStruct<'a> {
    pub fn new(struct_name: &'a str, index: Option<u8>) -> Self {
        Self {
            struct_index: index.unwrap_or_default(),
            struct_name_as_bytes: struct_name.as_bytes(),
            fields: HashMap::new(),
            num_fields: 0,
        }
    }

    pub fn add_field(
        &mut self,
        field_type: impl Into<FieldType<'a>>,
        field_name: &'a str,
        field_rules: FieldRules,
    ) {
        let field_index = self.num_fields;

        let field = VTableField::new(
            self.struct_index,
            field_type.into(),
            field_index,
            field_name,
            field_rules,
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

    pub fn field_rules_by_index(&self, index: &FieldIndex) -> Result<&FieldRules, Error> {
        self.field_by_index(index).map(|field| &field.field_rules)
    }

    pub fn struct_name_as_string(&self) -> Result<String, Error> {
        let name = String::from_utf8(self.struct_name_as_bytes.to_vec())?;
        Ok(name)
    }
}
