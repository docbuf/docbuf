use std::ops::Deref;

use super::*;

use serde_derive::{Deserialize, Serialize};

/// Average field size in bytes, used to estimate the size of the vtable.
/// This can be used for pre-allocating memory or page sizes for the DocBuf
/// document.
pub const AVG_FIELD_SIZE_IN_BYTES: u8 = u8::MAX;

/// Default entires per page
pub const DEFAULT_ENTRIES_PER_PAGE: u8 = u8::MAX;

const HASH_PRIME_CONST: u16 = 5;

/// Total number of items in the vtable.
pub type VTableNumItems = VTableItemIndex;

/// DocBuf documents can have up to 255 * 255 (u16::MAX, 65025) fields.
pub type VTableNumFields = u16;

/// VTable Root Item Name
pub type VTableRootItemName = String; //  = &'a str;

/// VTable Namespace
pub type VTableNamespace = String; //  = &'a str;

/// VTable Id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VTableId([u8; 8]);

impl Into<[u8; 8]> for &VTableId {
    fn into(self) -> [u8; 8] {
        self.0
    }
}

impl AsRef<[u8]> for VTableId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for VTableId {
    type Target = [u8; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&VTableId> for VTableId {
    fn from(src: &VTableId) -> Self {
        src.to_owned()
    }
}

impl From<&VTable> for VTableId {
    fn from(src: &VTable) -> Self {
        src.id().to_owned()
    }
}

impl From<&[u8; 8]> for VTableId {
    fn from(src: &[u8; 8]) -> Self {
        Self(*src)
    }
}

impl From<&[u8]> for VTableId {
    fn from(src: &[u8]) -> Self {
        let mut id = [0; 8];
        id.copy_from_slice(src);
        Self(id)
    }
}

impl From<u64> for VTableId {
    fn from(src: u64) -> Self {
        Self(src.to_le_bytes())
    }
}

impl From<VTableId> for u64 {
    fn from(src: VTableId) -> u64 {
        u64::from_le_bytes(src.0)
    }
}

impl From<[u8; 8]> for VTableId {
    fn from(src: [u8; 8]) -> Self {
        Self(src)
    }
}

impl Into<[u8; 8]> for VTableId {
    fn into(self) -> [u8; 8] {
        self.0
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VTable {
    pub namespace: VTableNamespace,
    /// The root name of the vtable.
    pub root: VTableRootItemName,
    pub items: VTableItems,
    pub num_items: VTableNumItems,
    /// Total number of fields in the vtable.
    pub num_fields: VTableNumFields,
}

impl std::fmt::Display for VTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl VTable {
    pub fn new(namespace: VTableNamespace, root: VTableRootItemName) -> Self {
        Self {
            namespace,
            root,
            items: VTableItems::new(),
            num_items: 0,
            num_fields: 0,
        }
    }

    #[inline]
    /// Serialize the vtable into a byte buffer.
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        // Clear the buffer
        buffer.clear();

        // Serialize the vtable
        let namespace_bytes = self.namespace.as_bytes();
        let namespace_len = namespace_bytes.len() as u8;
        buffer.push(namespace_len);
        buffer.extend_from_slice(namespace_bytes);

        let root_bytes = self.root.as_bytes();
        let root_len = root_bytes.len() as u8;
        buffer.push(root_len);
        buffer.extend_from_slice(root_bytes);

        buffer.push(self.num_items as u8);
        buffer.extend_from_slice(&self.num_fields.to_le_bytes());

        for item in self.items.iter() {
            item.write_to_buffer(buffer)?;
        }

        Ok(())
    }

    #[inline]
    pub fn read_from_buffer(buffer: &mut Vec<u8>) -> Result<Self, Error> {
        let namespace_len = buffer.remove(0);
        let namespace = String::from_utf8(buffer.drain(0..namespace_len as usize).collect())?;

        let root_len = buffer.remove(0);
        let root = String::from_utf8(buffer.drain(0..root_len as usize).collect())?;

        let num_items = buffer.remove(0);
        let num_fields = u16::from_le_bytes([buffer[0], buffer[1]]);
        buffer.drain(0..2);

        let mut vtable = Self::new(namespace, root);

        vtable.num_items = num_items;
        vtable.num_fields = num_fields;

        for _ in 0..num_items {
            let item = VTableItem::read_from_buffer(buffer)?;

            vtable.items.0.push(item);
        }

        Ok(vtable)
    }

    #[cfg(feature = "std")]
    #[inline]
    /// Read VTable from a file.
    pub fn from_file(path: impl Into<std::path::PathBuf>) -> Result<Self, Error> {
        use std::io::Read;

        let path = path.into();
        let mut file = std::fs::File::open(&path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Self::read_from_buffer(&mut buf)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.write_to_buffer(&mut buf)?;
        Ok(buf)
    }

    #[cfg(feature = "std")]
    #[inline]
    /// Write VTable to a file.
    pub fn to_file(&self, path: impl Into<std::path::PathBuf>) -> Result<(), Error> {
        use std::io::Write;

        let path = path.into();
        let mut file = std::fs::File::create(&path)?;
        let mut buf = Vec::new();
        self.write_to_buffer(&mut buf)?;
        file.write_all(&buf)?;
        Ok(())
    }

    #[inline]
    pub fn add_item(&mut self, vtable_item: VTableItem) {
        match vtable_item {
            VTableItem::Struct(vtable_struct) => {
                self.add_struct(vtable_struct);
            }
        }
    }

    #[inline]
    pub fn add_struct(&mut self, vtable_struct: VTableStruct) {
        let mut vtable_struct = vtable_struct;
        vtable_struct.set_item_index(self.num_items);
        self.num_fields += vtable_struct.num_fields as u16;
        self.items.add_struct(vtable_struct);
        self.num_items += 1;
    }

    #[inline]
    pub fn merge_vtable(&mut self, vtable: &'static VTable) {
        for vtable_item in vtable.items.iter() {
            if !self.items.0.contains(vtable_item) {
                match vtable_item {
                    VTableItem::Struct(vtable_struct) => {
                        self.add_struct(vtable_struct.to_owned());
                    }
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

    #[inline]
    /// Get the number of fields in an item, given its index.
    pub fn num_fields_by_index(&self, index: VTableItemIndex) -> Result<u8, Error> {
        match self.item_by_index(index)? {
            VTableItem::Struct(vtable_struct) => Ok(vtable_struct.num_fields),
        }
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

    #[inline]
    pub fn get_struct_item_index_by_name(&self, name: &str) -> Option<u8> {
        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.name == name {
                        return Some(vtable_struct.item_index);
                    }
                }
            }
        }

        None
    }

    // Return the field from the current item index and field index
    #[inline]
    pub fn get_item_field_by_index(
        &self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
    ) -> Result<&VTableField, Error> {
        // println!("get_item_field_by_index");
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

    /// Simple hash tagger. This is used to generate a 16-bit hash tag from a string slice.
    /// Collisions will occur with observations of 2^8 + 1 samples.
    #[inline]
    pub fn hash_tag(tag: &str) -> [u8; 2] {
        let bytes = tag.as_bytes();
        let len = bytes.len() as u16;

        bytes
            .into_iter()
            .enumerate()
            .fold(u16::MIN, |mut acc, (i, b)| {
                acc = acc
                    .wrapping_add(*b as u16 + len + i as u16)
                    .wrapping_mul(HASH_PRIME_CONST);

                acc
            })
            .to_le_bytes()
    }

    /// Return all fields in the vtable, including fields from all items.
    #[inline]
    pub fn fields(&self) -> VTableFields {
        let fields = self
            .items
            .0
            .iter()
            .map(|item| match item {
                VTableItem::Struct(vtable_struct) => vtable_struct.fields.0.clone(),
            })
            .flatten()
            .collect::<Vec<VTableField>>();

        VTableFields(fields)
    }

    #[inline]
    pub fn namespace_tag(&self) -> [u8; 2] {
        Self::hash_tag(&self.namespace)
    }

    #[inline]
    pub fn root_tag(&self) -> [u8; 2] {
        Self::hash_tag(&self.root)
    }

    #[inline]
    /// Return the vtable identifier as a 8 byte array.
    pub fn id(&self) -> &'static VTableId {
        static VTABLE_ID: ::std::sync::OnceLock<VTableId> = ::std::sync::OnceLock::new();

        VTABLE_ID.get_or_init(|| {
            let mut id = [0u8; 8];
            id[0..2].copy_from_slice(&self.namespace_tag());
            id[2..4].copy_from_slice(&self.root_tag());
            id[4] = (self.namespace.len() + self.root.len()) as u8;
            id[5] = self.num_items;
            id[6..8].copy_from_slice(&self.num_fields.to_le_bytes());

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
        vec![0; self.avg_size() * 2]
    }

    #[inline]
    pub fn num_offsets(&self) -> u16 {
        // Add 1 to account for the root item offset
        // Substract the number of items to account for the flat array of offsets
        self.num_fields + 1 - self.num_items as u16
    }

    #[inline]
    pub fn offset_len(&self) -> usize {
        self.num_offsets() as usize * VTABLE_FIELD_OFFSET_SIZE_BYTES
    }

    #[inline]
    /// Check the byte offsets against the vtable.
    pub fn check_offsets(&self, offsets: &[u8]) -> Result<(), Error> {
        let num_offsets = (offsets.len() / VTABLE_FIELD_OFFSET_SIZE_BYTES) as u16;

        if num_offsets != self.num_offsets() {
            return Err(Error::InvalidOffsets(self.num_offsets(), num_offsets));
        }

        Ok(())
    }
}

impl PartialEq for VTable {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace
            && self.root == other.root
            && self.items == other.items
            && self.num_items == other.num_items
            && self.num_fields == other.num_fields
    }
}
