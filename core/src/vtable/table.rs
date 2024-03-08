// use std::collections::{HashMap, HashSet};

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
        Self(Vec::with_capacity(256))
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
            items: VTableItems::new(),
            num_items: 0,
        }
    }

    #[inline]
    pub fn add_struct(&mut self, vtable_struct: VTableStruct<'a>) {
        let mut vtable_struct = vtable_struct;
        vtable_struct.set_item_index(self.num_items);
        self.items.add_struct(vtable_struct);
        self.num_items += 1;
    }

    #[inline]
    pub fn merge_vtable(&mut self, vtable: &'static VTable) {
        for vtable_item in vtable.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    self.add_struct(vtable_struct.to_owned());
                }
            }
        }
    }

    #[inline]
    pub fn item_by_index(&self, index: VTableItemIndex) -> Result<&VTableItem, Error> {
        if index as usize >= self.items.len() {
            return Err(Error::ItemNotFound);
        }

        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.item_index == index {
                        return Ok(vtable_item);
                    }
                }
            }
        }

        Err(Error::ItemNotFound)
    }

    // Return the struct name from the struct index
    #[inline]
    pub fn struct_by_index(&self, index: VTableItemIndex) -> Result<&VTableStruct, Error> {
        if index as usize >= self.items.len() {
            return Err(Error::StructNotFound);
        }

        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.item_index == index {
                        return Ok(vtable_struct);
                    }
                }
            }
        }

        Err(Error::StructNotFound)
    }

    // Return the struct index from the struct name
    #[inline]
    pub fn struct_by_name(&self, name: &str) -> Result<&VTableStruct, Error> {
        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.struct_name == name {
                        return Ok(vtable_struct);
                    }
                }
            }
        }

        Err(Error::StructNotFound)
    }

    // Return the field from the vtable by struct index and field index
    #[inline]
    pub fn get_struct_field_by_index(
        &self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
    ) -> Result<&VTableField, Error> {
        let vtable_struct = self.struct_by_index(item_index)?;
        vtable_struct.field_by_index(&field_index)
    }

    // Return the field from the current item index and field index
    #[inline]
    pub fn get_item_field_by_index(
        &self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
    ) -> Result<&VTableField, Error> {
        self.item_by_index(item_index)
            .and_then(|vtable_item| match vtable_item {
                VTableItem::Struct(vtable_struct) => vtable_struct
                    .field_by_index(&field_index)
                    .map_err(|_| Error::FieldNotFound),
            })
    }
}
