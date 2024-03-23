pub mod structs;

use super::*;
pub use structs::*;

use serde_derive::{Deserialize, Serialize};

pub type VTableItemIndex = u8;

/// VTable Items are the only structs or enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VTableItem {
    Struct(VTableStruct),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VTableItems(pub Vec<VTableItem>);

impl VTableItems {
    pub fn new() -> Self {
        Self(Vec::with_capacity(256))
    }

    pub fn add_struct(&mut self, vtable_struct: VTableStruct) {
        self.0.push(VTableItem::Struct(vtable_struct));
    }

    pub fn iter(&self) -> std::slice::Iter<'_, VTableItem> {
        self.0.iter()
    }

    // Inner values
    pub fn inner(&self) -> &Vec<VTableItem> {
        &self.0
    }

    // Inner values mutable
    pub fn inner_mut(&mut self) -> &mut Vec<VTableItem> {
        &mut self.0
    }

    // Returns the length of items
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
