use std::collections::HashMap;

use super::*;

/// VTable Items are the only structs or enums
#[derive(Debug, Clone)]
pub enum VTableItem<'a> {
    Struct(VTableStruct<'a>),
}

#[derive(Debug, Clone)]
pub struct VTableItems<'a>(pub Vec<VTableItem<'a>>);

impl<'a> VTableItems<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_struct(&mut self, vtable_struct: VTableStruct<'a>) {
        self.0.push(VTableItem::Struct(vtable_struct));
    }

    pub fn iter(&self) -> std::slice::Iter<'_, VTableItem<'a>> {
        self.0.iter()
    }

    // Inner values
    pub fn inner(&self) -> &Vec<VTableItem<'a>> {
        &self.0
    }

    // Inner values mutable
    pub fn inner_mut(&mut self) -> &mut Vec<VTableItem<'a>> {
        &mut self.0
    }

    // Returns the length of items
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub type VTableItemIndex = u8;

#[derive(Debug, Clone)]
pub struct VTable<'a> {
    pub items: VTableItems<'a>,
    pub num_items: VTableItemIndex,
}

impl std::fmt::Display for VTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> VTable<'a> {
    pub fn new() -> Self {
        Self {
            // structs: HashMap::new(),
            items: VTableItems::new(),
            num_items: 0,
        }
    }

    pub fn add_struct(&mut self, vtable_struct: VTableStruct<'a>) {
        let mut vtable_struct = vtable_struct;
        vtable_struct.set_item_index(self.num_items);
        self.items.add_struct(vtable_struct);
        self.num_items += 1;
    }

    pub fn merge_vtable(&mut self, vtable: &'static VTable) {
        for vtable_item in vtable.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    self.add_struct(vtable_struct.to_owned());
                }
                _ => {}
            }
        }
    }

    // Return the struct name from the struct index
    pub fn struct_by_index(&self, index: VTableItemIndex) -> Result<&VTableStruct, Error> {
        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.item_index == index {
                        return Ok(vtable_struct);
                    }
                }
                _ => {}
            }
        }

        Err(Error::StructNotFound)
    }

    // Return the struct index from the struct name
    pub fn struct_by_name(&self, name: impl AsRef<[u8]>) -> Result<&VTableStruct, Error> {
        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.struct_name_as_bytes == name.as_ref() {
                        return Ok(vtable_struct);
                    }
                }
                _ => {}
            }
        }

        Err(Error::StructNotFound)
    }

    // Return the field from the vtable by struct index and field index
    pub fn get_struct_field_by_index(
        &self,
        item_index: VTableItemIndex,
        field_index: FieldIndex,
    ) -> Result<&VTableField, Error> {
        let vtable_struct = self.struct_by_index(item_index)?;
        vtable_struct.field_by_index(&field_index)
    }

    pub fn parse_raw_values(&self, input: &[u8]) -> Result<DocBufRawValues, Error> {
        let mut current_item_index = 0;
        let mut current_field_index = 0;

        let mut data = DocBufRawValues::new();
        let mut input = input.to_vec();

        while !input.is_empty() {
            match self.get_struct_field_by_index(current_item_index, current_field_index) {
                Ok(field) => {
                    let field_data = field.decode(&mut input)?;

                    if !field_data.is_empty() {
                        data.insert_value(current_item_index, current_field_index, field_data);
                    }

                    current_field_index += 1;
                }
                _ => {
                    // This will error when the item_index reaches the u8 max value (255)
                    // if there is an unhandled error and the input data is not read
                    current_item_index += 1;
                    current_field_index = 0;
                }
            }
        }

        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct DocBufRawValues(HashMap<VTableItemIndex, HashMap<FieldIndex, Vec<u8>>>);

impl DocBufRawValues {
    pub fn new() -> Self {
        DocBufRawValues(HashMap::new())
    }

    pub fn insert_value(
        &mut self,
        item_index: VTableItemIndex,
        field_index: FieldIndex,
        value: Vec<u8>,
    ) {
        self.0
            .entry(item_index)
            .or_insert_with(HashMap::new)
            .insert(field_index, value);
    }

    pub fn get(&self, item_index: VTableItemIndex, field_index: FieldIndex) -> Option<&Vec<u8>> {
        self.0.get(&item_index)?.get(&field_index)
    }

    // Check if the raw values is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // Remove the value from the raw values
    pub fn remove(
        &mut self,
        item_index: VTableItemIndex,
        field_index: FieldIndex,
    ) -> Option<Vec<u8>> {
        let structs = self.0.get_mut(&item_index)?;
        let value = structs.remove(&field_index);

        // Remove the struct if it's empty
        if structs.is_empty() {
            self.0.remove(&item_index);
        }

        value
    }
}
