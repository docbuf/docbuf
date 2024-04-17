use super::*;

use crate::DocBufDbMngr;
use crate::Error;

use docbuf_core::traits::DocBuf;
use docbuf_core::vtable::*;

use std::collections::HashSet;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct DocBufDbManager {
    /// Configuration for the database
    pub config: DocBufDbConfig,
}

impl DocBufDbManager {
    pub fn from_config(config_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let config = DocBufDbConfig::load(config_path.into())?;

        Ok(Self { config })
    }

    pub fn write_docbuf(
        &self,
        id: &[u8; 16],
        buffer: Vec<u8>,
        vtable_id: &[u8; 8],
        partition_key: &[u8; 16],
        offsets: Vec<u8>,
    ) -> Result<(), Error> {
        let mut partition = self.config.partition_file(vtable_id, partition_key)?;

        partition.write_docbuf(buffer, offsets)?;

        // let mut page_lock = self.vtable_page_lock(vtable, partition_key)?;

        // println!("VTable Page Lock: {:?}", page_lock);

        Ok(())
    }

    /// Read vtable file from the database.
    pub fn read_vtable(&self, vtable_id: &[u8; 8]) -> Result<VTable, Error> {
        let vtable = self.config.read_vtable(vtable_id)?;

        Ok(vtable)
    }

    pub fn write_vtable(&self, vtable: &VTable) -> Result<(), Error> {
        // Crate the vtable file
        self.config.write_vtable(vtable)?;

        Ok(())
    }
}

impl DocBufDbMngr for DocBufDbManager {
    /// The predicate type used for querying the database.
    type Predicate = ();

    /// Write a document into the database.
    /// This will return the document id.
    fn insert<D: DocBuf>(
        &self,
        doc: &D,
        partition_key: PartitionKey,
    ) -> Result<docbuf_core::deps::uuid::Uuid, Error> {
        let id = doc.uuid()?;

        // Return the vtable for the document.
        let vtable = D::vtable()?;

        // Allocate a buffer for the document.
        let mut buffer = vtable.alloc_buf();

        //
        let offsets = doc.to_docbuf(&mut buffer)?;

        // Write the document buffer to the database.
        self.write_docbuf(
            id.as_bytes(),
            buffer,
            &vtable.id(),
            partition_key.deref(),
            offsets.to_vec(),
        )?;

        // if not, return an error.
        // if !self.vtables.contains(vid.as_slice()) {
        //     // Create the vtable collection.

        //     // return Err(Error::Database(Error::VTableIdNotFound));
        // }

        // println!("DocBuf VTable Id: {:?}", vid);

        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Return all documents in the database.
    fn all<D: DocBuf>(&self, _doc: &D) -> Result<Vec<D>, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Read documents in the database given a predicate.
    fn find<D: DocBuf>(&self, _doc: &D, _predicate: Self::Predicate) -> Result<Vec<D>, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Get a document from the database.
    fn get<D: DocBuf>(&self, _id: docbuf_core::deps::uuid::Uuid) -> Result<D, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Update a document in the database.
    fn update<D: DocBuf>(&self, _doc: &D) -> Result<(), Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Delete a document from the database.
    fn delete<D: DocBuf>(&self, _doc: D) -> Result<D, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Return the number of documents in the database.
    fn count<D: DocBuf>(&self, _doc: &D) -> Result<usize, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Return the number of documents in the database given a predicate.
    fn count_where(
        &self,
        _vtable_id: &VTableId,
        _predicate: Self::Predicate,
    ) -> Result<usize, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    fn vtable_ids(&self) -> Result<&HashSet<VTableId>, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }
}
