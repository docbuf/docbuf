use super::*;

use serde_derive::{Deserialize, Serialize};

/// Average field size in bytes, used to estimate the size of the vtable.
/// This can be used for pre-allocating memory or page sizes for the DocBuf
/// document.
pub const AVG_FIELD_SIZE_IN_BYTES: u8 = u8::MAX;

/// Default entires per page
pub const DEFAULT_ENTRIES_PER_PAGE: u8 = u8::MAX;

/// Total number of items in the vtable.
pub type VTableNumItems = VTableItemIndex;

/// DocBuf documents can have up to 255 * 255 (u16::MAX, 65025) fields.
pub type VTableNumFields = u16;

/// VTable Root Item Name
pub type VTableRootItemName<'a> = &'a str;

/// VTable Id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VTableId([u8; 8]);

impl AsRef<[u8]> for VTableId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for VTableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_hex())
    }
}

impl VTableId {
    pub fn new(id: [u8; 8]) -> Self {
        Self(id)
    }

    pub fn from_hex(hex: &str) -> Result<Self, Error> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 8 {
            return Err(Error::InvalidVTableId(
                "VTableId must be 8 characters long".to_string(),
            ));
        }
        let mut id = [0; 8];
        id.copy_from_slice(&bytes);
        Ok(Self(id))
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct VTable<'a> {
    /// The root name of the vtable.
    pub root: VTableRootItemName<'a>,
    pub items: VTableItems<'a>,
    pub num_items: VTableNumItems,
    /// Total number of fields in the vtable.
    pub num_fields: VTableNumFields,
}

impl std::fmt::Display for VTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> VTable<'a> {
    pub fn new(root: VTableRootItemName<'a>) -> Self {
        Self {
            root,
            items: VTableItems::new(),
            num_items: 0,
            num_fields: 0,
        }
    }

    #[inline]
    pub fn add_struct(&mut self, vtable_struct: VTableStruct<'a>) {
        let mut vtable_struct = vtable_struct;
        vtable_struct.set_item_index(self.num_items);
        self.num_fields += vtable_struct.num_fields as u16;
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
                    if vtable_struct.name == name {
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

    #[inline]
    pub fn get_field_by_offset_index(
        &self,
        index: VTableFieldOffsetIndex,
    ) -> Result<&VTableField, Error> {
        self.get_item_field_by_index(index.0, index.1)
    }

    #[inline]
    pub fn root_tag(&self) -> [u8; 5] {
        let rb = self.root.as_bytes();
        let mut tag = [0u8; 5];
        let rlen = rb.len();

        if rlen <= 5 {
            for i in 0..rlen {
                tag[i] = rb[i];
            }
        } else {
            tag[0] = rb[0];
            tag[1] = rb[1];

            if rlen % 2 == 0 {
                tag[2] = rb[rlen / 2];
            } else {
                tag[2] = rb[rlen / 2 + 1];
            }

            tag[3] = rb[rlen - 2];
            tag[4] = rb[rlen - 1];
        }

        tag
    }

    #[inline]
    /// Return the vtable identifier as a 8 byte array.
    pub fn id(&self) -> &'static VTableId {
        static VTABLE_ID: ::std::sync::OnceLock<VTableId> = ::std::sync::OnceLock::new();

        VTABLE_ID.get_or_init(|| {
            let mut id = [0u8; 8];
            let tag = self.root_tag();

            // Indicate the tag
            id[0] = tag[0];
            id[1] = tag[1];
            id[2] = tag[2];
            id[3] = tag[3];
            id[4] = tag[4];

            // Indicate the number of items
            id[5] = self.num_items;

            // Indicate the number of fields
            let num_fields_bytes = self.num_fields.to_le_bytes();
            id[6] = num_fields_bytes[0];
            id[7] = num_fields_bytes[1];

            VTableId::new(id)
        })
    }

    /// Average size of the vtable will be less than u24::MAX bytes,
    /// roughly 16 MB. This is used by the DocBuf database to determine
    /// the suggested page size of the vtable when persisting to disk.
    /// This is also helpful for allocating a memory buffer for the vtable.
    #[inline]
    pub fn avg_size(&self) -> usize {
        self.num_fields as usize * AVG_FIELD_SIZE_IN_BYTES as usize
    }

    /// Return the suggested page size of the vtable.
    #[inline]
    pub fn page_size(&self, num_entries: Option<usize>) -> usize {
        self.avg_size() * num_entries.unwrap_or(DEFAULT_ENTRIES_PER_PAGE as usize)
    }

    /// Allocate a re-usable (owned) buffer for for the vtable based on the average size.
    #[inline]
    pub fn alloc_buf(&self) -> Vec<u8> {
        Vec::with_capacity(self.avg_size())
    }
}
