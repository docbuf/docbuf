use super::*;

use crate::DocBufDbMngr;
use crate::Error;

use docbuf_core::traits::DocBuf;
use docbuf_core::vtable::*;

use std::collections::HashSet;
// use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct DocBufDbManager {
    /// Configuration for the database
    pub config: DocBufDbConfig,
    /// Lock file for the database
    pub lock: DocBufDbLock,
}

impl DocBufDbManager {
    pub fn from_config(config_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let config = DocBufDbConfig::load(config_path.into())?;

        let lock = DocBufDbLock::load(config.directory()?.clone())?;

        Ok(Self { config, lock })
    }

    pub fn write_docbuf(
        &self,
        partition_key: PartitionKey,
        id: docbuf_core::uuid::Uuid,
        buffer: Vec<u8>,
        vtable: &VTable,
        offsets: VTableFieldOffsets,
    ) -> Result<(), Error> {
        let mut page_lock = self.vtable_page_lock(vtable, partition_key)?;

        println!("VTable Page Lock: {:?}", page_lock);

        Ok(())
    }

    // TODO: Add a file guard to the lock file.
    pub fn vtable_page_lock(
        &self,
        vtable: &VTable,
        partition_key: PartitionKey,
    ) -> Result<DocBufDbVTableLockFile, Error> {
        println!("Loading vtable page lock...");
        let path = self.config.vtable_lock_file(vtable.id())?;
        println!("Lock Path: {:?}", path);
        match DocBufDbVTableLockFile::load(&path) {
            Ok(lock) => Ok(lock),
            Err(_) => {
                println!("Creating vtable page lock...");

                // Ensure the vtable directory exists.
                std::fs::create_dir_all(self.config.vtable_directory(vtable.id())?)?;

                let mut lock = DocBufDbVTableLockFile::new(path, vtable)?;

                // Add the page metadata to the lock file.
                lock.new_page_metadata()?;

                // Save the lock file, if it doesn't exist, before returning it.
                lock.save()?;

                Ok(lock)
            }
        }
    }

    pub fn check_vtable(&self, vtable: &VTable) -> Result<(), Error> {
        // Set up the vtable if it doesn't exist.
        if !self.lock.state.vtables.contains(vtable.id()) {
            self.create_vtable(vtable)?;
        }

        Ok(())
    }

    pub fn create_vtable(&self, vtable: &VTable) -> Result<(), Error> {
        // Create the vtable folder.
        std::fs::create_dir_all(self.config.vtable_directory(vtable.id())?)?;
        // Create the vtable lock file.
        std::fs::File::create(self.config.vtable_lock_file(vtable.id())?)?;

        Ok(())
    }

    /// Create a vtable page for the given vtable.
    pub fn create_vtable_page(&self, vtable: &VTable) -> Result<(), Error> {
        unimplemented!("DocBufDbMngr method not implemented");
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
    ) -> Result<docbuf_core::uuid::Uuid, Error> {
        let id = doc.uuid()?;

        // Return the vtable for the document.
        let vtable = D::vtable()?;

        // Allocate a buffer for the document.
        let mut buffer = vtable.alloc_buf();

        let offsets = doc.to_docbuf(&mut buffer)?;

        // Write the document buffer to the database.
        self.write_docbuf(partition_key, id, buffer, &vtable, offsets)?;

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
    fn get<D: DocBuf>(&self, _id: docbuf_core::uuid::Uuid) -> Result<D, Error> {
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
