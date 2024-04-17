pub mod structs;

use super::*;
pub use structs::*;

use serde_derive::{Deserialize, Serialize};

pub type VTableItemIndex = u8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VTableItem {
    Struct(VTableStruct),
}

impl Into<u8> for &VTableItem {
    fn into(self) -> u8 {
        match self {
            VTableItem::Struct(_) => 0,
        }
    }
}

impl From<u8> for VTableItem {
    fn from(value: u8) -> Self {
        match value {
            0 => VTableItem::Struct(VTableStruct::default()),
            _ => unimplemented!("VTableItem::from"),
        }
    }
}

impl VTableItem {
    #[inline]
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        buffer.push(self.into());

        match self {
            VTableItem::Struct(vtable_struct) => {
                vtable_struct.write_to_buffer(buffer)?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn read_from_buffer(buffer: &mut Vec<u8>) -> Result<Self, Error> {
        let value: Self = buffer.remove(0).into();

        let item = match value {
            VTableItem::Struct(_) => {
                let s = VTableStruct::read_from_buffer(buffer)?;
                Self::Struct(s)
            }
        };

        Ok(item)
    }
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

    /// Write to buffer
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        for item in self.0.iter() {
            item.write_to_buffer(buffer)?;
        }

        Ok(())
    }
}

impl PartialEq for VTableItems {
    fn eq(&self, other: &Self) -> bool {
        for (item, other_item) in self.0.iter().zip(other.0.iter()) {
            if item != other_item {
                return false;
            }
        }

        true
    }
}

impl PartialEq for VTableItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VTableItem::Struct(a), VTableItem::Struct(b)) => a == b,
        }
    }
}
