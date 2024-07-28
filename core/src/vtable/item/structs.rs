use super::*;

use serde_derive::{Deserialize, Serialize};

pub type StructName = String; //  = &'a str;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
        let field = self
            .fields
            .find_field_by_name(name)
            .ok_or(Error::FieldNotFound)?;

        Ok(field)
    }

    // Return the field by index from the struct
    #[inline]
    pub fn field_by_index(&self, index: &VTableFieldIndex) -> Result<&VTableField, Error> {
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

    #[inline]
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        // Item index it belongs to
        buffer.push(self.item_index);

        // Name of the struct
        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len() as u8;
        buffer.push(name_len);
        buffer.extend_from_slice(name_bytes);

        // Number of fields in the struct
        buffer.push(self.num_fields);

        // Serialize the fields
        for field in self.fields.iter() {
            field.write_to_buffer(buffer)?;
        }

        Ok(())
    }

    #[inline]
    pub fn read_from_buffer(buffer: &mut Vec<u8>) -> Result<Self, Error> {
        let item_index = buffer.remove(0);
        let name_len = buffer.remove(0) as usize;
        let name = String::from_utf8(buffer.drain(0..name_len).collect())?;

        let num_fields = buffer.remove(0);

        let mut fields = VTableFields::new();
        for _ in 0..num_fields {
            let field = VTableField::read_from_buffer(buffer)?;
            fields.add_field(field);
        }

        Ok(Self {
            item_index,
            name,
            fields,
            num_fields,
        })
    }
}

impl PartialEq for VTableStruct {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.fields == other.fields
            && self.num_fields == other.num_fields
            && self.item_index == other.item_index
    }
}
