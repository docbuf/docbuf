use crate::Error;

use std::collections::BTreeSet;
// use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Range;
use std::path::PathBuf;

use docbuf_core::vtable::{VTable, VTableId};
use serde::{Deserialize, Serialize};

pub type VTableIdAsHex = String;
pub type PageSize = usize;
pub type PageIndex = u32;
pub type PageCount = u32;
pub type PageEndOffset = usize;
pub type PageAvailableSpace = usize;
pub type PageDocumentCount = usize;
pub type PageOffset = Range<usize>;
pub type DocBufDbDocIdBytes = [u8; 28];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocBufDbDocId {
    pub vtable_id: VTableId,
    pub page_index: PageIndex,
    pub page_offset: PageOffset,
}

impl Into<DocBufDbDocIdBytes> for DocBufDbDocId {
    fn into(self) -> DocBufDbDocIdBytes {
        let mut bytes = [0; 28];

        bytes[0..8].copy_from_slice(&self.vtable_id.as_ref());
        bytes[8..12].copy_from_slice(&self.page_index.to_le_bytes());
        bytes[12..20].copy_from_slice(&self.page_offset.start.to_le_bytes());
        bytes[20..28].copy_from_slice(&self.page_offset.end.to_le_bytes());

        bytes
    }
}

impl From<DocBufDbDocIdBytes> for DocBufDbDocId {
    fn from(bytes: DocBufDbDocIdBytes) -> Self {
        Self::from(bytes.as_slice())
    }
}

impl From<&[u8]> for DocBufDbDocId {
    fn from(bytes: &[u8]) -> Self {
        let mut vtable_id_bytes = [0; 8];
        vtable_id_bytes.copy_from_slice(&bytes[0..8]);
        let vtable_id = VTableId::new(vtable_id_bytes);

        let mut page_index_bytes = [0; 4];
        page_index_bytes.copy_from_slice(&bytes[8..12]);
        let page_index = PageIndex::from_le_bytes(page_index_bytes);

        let mut page_offset_start_bytes = [0; 8];
        page_offset_start_bytes.copy_from_slice(&bytes[12..20]);
        let page_offset_start = usize::from_le_bytes(page_offset_start_bytes);

        let mut page_offset_end_bytes = [0; 8];
        page_offset_end_bytes.copy_from_slice(&bytes[20..28]);
        let page_offset_end = usize::from_le_bytes(page_offset_end_bytes);

        Self {
            vtable_id,
            page_index,
            page_offset: page_offset_start..page_offset_end,
        }
    }
}

impl DocBufDbDocId {
    pub fn new(vtable_id: VTableId, page_index: PageIndex, page_offset: PageOffset) -> Self {
        Self {
            vtable_id,
            page_index,
            page_offset,
        }
    }
}

/// A page in the document buffer database.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocBufDbVTablePageMetadata {
    /// The page index for the vtable
    pub index: PageIndex,
    /// The number of documents in the page.
    pub count: PageCount,
    /// The end offset of the last document in the page.
    pub end: PageEndOffset,
    /// Available space in the page in bytes.
    /// (size - end)
    pub available: PageAvailableSpace,
}

impl DocBufDbVTablePageMetadata {
    pub fn new(index: PageIndex, size: PageSize) -> Self {
        Self {
            index,
            available: size,
            count: 0,
            end: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocBufDbVTableLockFile {
    #[serde(skip)]
    file: Option<File>,
    /// The virtual tabe.
    pub vtable: VTable,
    /// The default size of the pages in bytes.
    pub size: PageSize,
    /// The pages in the vtable
    pub pages: Vec<DocBufDbVTablePageMetadata>,
    /// Document IDs in the vtable.
    pub doc_ids: BTreeSet<DocBufDbDocIdBytes>,
}

impl DocBufDbVTableLockFile {
    pub fn new(path: impl Into<PathBuf>, vtable: &VTable) -> Result<Self, Error> {
        Ok(Self {
            file: Some(File::create(&path.into())?),
            vtable: vtable.to_owned(),
            size: vtable.page_size(None),
            pages: Vec::new(),
            doc_ids: BTreeSet::new(),
        })
    }

    pub fn file(&self) -> Result<&File, Error> {
        self.file
            .as_ref()
            .ok_or(Error::PageLockError(self.vtable.id().as_hex()))
    }

    /// Deserialize the lock file from toml.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let mut file = File::create(&path.into())?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let metadata = toml::from_str(buffer.as_str())?;

        Ok(Self {
            file: Some(file),
            ..metadata
        })
    }

    /// Save the lock file to toml.
    /// Overwrites the existing file.
    pub fn save(&self) -> Result<(), Error> {
        let toml = toml::to_string(&self)?;

        self.file()?.write_all(toml.as_bytes())?;

        Ok(())
    }

    /// Create a new page
    pub fn new_page_metadata(&mut self) -> Result<(), Error> {
        let page_index = self
            .pages
            .len()
            .try_into()
            .map_err(|_| Error::MaxPagesReached(self.vtable.id().as_hex()))?;

        self.pages
            .push(DocBufDbVTablePageMetadata::new(page_index, self.size));

        Ok(())
    }

    /// Retrieve the current available (last) page.
    pub fn current_page_metadata(&mut self) -> Option<&mut DocBufDbVTablePageMetadata> {
        self.pages.last_mut()
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}
