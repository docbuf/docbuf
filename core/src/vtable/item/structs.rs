use super::*;

use serde_derive::{Deserialize, Serialize};

pub type StructName = String; //  = &'a str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VTableStruct {
    pub item_index: VTableItemIndex,
    pub name: StructName,
    pub fields: VTableFields,
    pub num_fields: VTableFieldIndex,
}

impl VTableStruct {
    pub fn new(name: &str, index: Option<u8>) -> Self {
        Self {
            item_index: index.unwrap_or_default(),
            name: name.to_owned(),
            fields: VTableFields::new(),
            num_fields: 0,
        }
    }

    #[inline]
    pub fn add_field(
        &mut self,
        field_type: impl Into<VTableFieldType>,
        field_name: &str,
        field_rules: VTableFieldRules,
    ) {
        let field_index = self.num_fields;
        let field_type: VTableFieldType = field_type.into();

        let field = VTableField::new(
            self.item_index,
            field_type.clone(),
            field_index,
            field_name.to_owned(),
            field_rules,
        );
        self.fields.add_field(field);

        // If the field type is not a struct, increment the number of fields
        // if let VTableFieldType::Struct(_) = field_type {
        //     // Return early, do not increment the number of fields,
        //     // as the number of fields will be incremented when the struct is added
        //     return;
        // }

        self.num_fields += 1;
    }

    #[inline]
    pub fn set_item_index(&mut self, index: VTableItemIndex) {
        self.item_index = index;

        for field in self.fields.inner_mut() {
            field.item_index = self.item_index;
        }
    }

    // Return the field index from the struct
    #[inline]
    pub fn field_index_from_name(&self, name: &str) -> Result<VTableFieldIndex, Error> {
        Ok(self.field_by_name(name)?.index)
    }

    // Return the field by name from the struct
    #[inline]
    pub fn field_by_name(&self, name: &str) -> Result<&VTableField, Error> {
        // println!("Field By Name");
        self.fields
            .find_field_by_name(name)
            .ok_or(Error::FieldNotFound)
    }

    // Return the field by index from the struct
    #[inline]
    pub fn field_by_index(&self, index: &VTableFieldIndex) -> Result<&VTableField, Error> {
        // println!("Field By Index");
        for field in self.fields.iter() {
            if field.index == *index {
                return Ok(field);
            }
        }

        Err(Error::FieldNotFound)
    }

    #[inline]
    pub fn field_rules_by_index(
        &self,
        index: &VTableFieldIndex,
    ) -> Result<&VTableFieldRules, Error> {
        self.field_by_index(index).map(|field| &field.rules)
    }
}
