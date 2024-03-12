use super::*;

pub type StructName<'a> = &'a str;

#[derive(Debug, Clone)]
pub struct VTableStruct<'a> {
    pub item_index: VTableItemIndex,
    pub name: StructName<'a>,
    pub fields: VTableFields<'a>,
    pub num_fields: VTableFieldIndex,
}

impl<'a> VTableStruct<'a> {
    pub fn new(name: &'a str, index: Option<u8>) -> Self {
        Self {
            item_index: index.unwrap_or_default(),
            name,
            fields: VTableFields::new(),
            num_fields: 0,
        }
    }

    #[inline]
    pub fn add_field(
        &mut self,
        field_type: impl Into<VTableFieldType<'a>>,
        field_name: &'a str,
        field_rules: VTableFieldRules,
    ) {
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
