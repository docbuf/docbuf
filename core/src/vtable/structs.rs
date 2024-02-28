use super::*;

pub type StructName<'a> = &'a str;

#[derive(Debug, Clone)]
pub struct VTableStruct<'a> {
    pub item_index: VTableItemIndex,
    pub struct_name: StructName<'a>,
    pub fields: VTableFields<'a>,
    pub num_fields: FieldIndex,
}

impl<'a> VTableStruct<'a> {
    pub fn new(struct_name: &'a str, index: Option<u8>) -> Self {
        Self {
            item_index: index.unwrap_or_default(),
            struct_name,
            fields: VTableFields::new(),
            num_fields: 0,
        }
    }

    pub fn add_field(
        &mut self,
        field_type: impl Into<FieldType<'a>>,
        field_name: &'a str,
        field_rules: FieldRules,
    ) {
        // println!("Adding Field: {}", field_name);

        let field_index = self.num_fields;

        let field = VTableField::new(
            self.item_index,
            field_type.into(),
            field_index,
            field_name,
            field_rules,
        );
        self.fields.add_field(field);
        self.num_fields += 1;
    }

    pub fn set_item_index(&mut self, index: VTableItemIndex) {
        self.item_index = index;

        for field in self.fields.inner_mut() {
            field.item_index = self.item_index;
        }
    }

    // Return the field index from the struct
    pub fn field_index_from_name(&self, name: &str) -> Result<FieldIndex, Error> {
        Ok(self.field_by_name(name)?.field_index)
    }

    // Return the field by name from the struct
    pub fn field_by_name(&self, name: &str) -> Result<&VTableField, Error> {
        self.fields
            .find_field_by_name(name)
            .ok_or(Error::FieldNotFound)
    }

    // Return the field by index from the struct
    pub fn field_by_index(&self, index: &FieldIndex) -> Result<&VTableField, Error> {
        for field in self.fields.iter() {
            if field.field_index == *index {
                return Ok(field);
            }
        }

        Err(Error::FieldNotFound)
    }

    pub fn field_rules_by_index(&self, index: &FieldIndex) -> Result<&FieldRules, Error> {
        self.field_by_index(index).map(|field| &field.field_rules)
    }
}
