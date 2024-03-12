pub mod structs;

use super::*;
pub use structs::*;

pub type VTableItemIndex = u8;

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
