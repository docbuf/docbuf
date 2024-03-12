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

    pub fn check_vtable(&self, vtable: &VTable) -> Result<(), Error> {
        // Set up the vtable if it doesn't exist.
        if !self.lock.state.vtables.contains(vtable.id()) {
            self.create_vtable(vtable)?;
        }

        Ok(())
    }

    pub fn create_vtable(&self, vtable: &VTable) -> Result<(), Error> {
        let id = vtable.id().as_hex();

        // Create the vtable folder.
        let vtable_dir = self.config.directory()?.join("vtables").join(&id);

        std::fs::create_dir_all(&vtable_dir)?;

        // Create the initial vtable lock file.
        let vtable_lock = vtable_dir.join(format!("{}.lock", &id));
        std::fs::File::create(vtable_lock)?;

        // // Add the vtable to the lock file.
        // self.lock.state.vtables.insert(vtable.id().to_owned());
        // self.lock.save()?;

        Ok(())
    }
}

impl DocBufDbMngr for DocBufDbManager {
    /// The unique document id type.
    type DbDocId = [u8; 16];

    /// The predicate type used for querying the database.
    type Predicate = ();

    fn db_doc_id<D: DocBuf>(&self, _doc: &D) -> Result<Option<Self::DbDocId>, Error> {
        let vid = D::vtable()?.id();

        println!("DocBuf VTable Id: {:?}", vid);

        unimplemented!("DocBufDbMngr method not implemented")
    }

    /// Write a document into the database.
    /// This will return the document id.
    fn db_insert<D: DocBuf>(&self, _doc: &D) -> Result<Self::DbDocId, Error> {
        // Check if the vtable is in the database,
        self.check_vtable(D::vtable()?)?;
        // if not, return an error.
        // if !self.vtables.contains(vid.as_slice()) {
        //     // Create the vtable collection.

        //     // return Err(Error::Database(Error::VTableIdNotFound));
        // }

        // println!("DocBuf VTable Id: {:?}", vid);

        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Return all documents in the database.
    fn db_all<D: DocBuf>(&self, _doc: &D) -> Result<Vec<D>, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Read documents in the database given a predicate.
    fn db_find<D: DocBuf>(&self, _doc: &D, _predicate: Self::Predicate) -> Result<Vec<D>, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Get a document from the database.
    fn db_get<D: DocBuf>(&self, _id: Self::DbDocId) -> Result<D, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Update a document in the database.
    fn db_update<D: DocBuf>(&self, _doc: &D) -> Result<(), Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Delete a document from the database.
    fn db_delete<D: DocBuf>(&self, _doc: D) -> Result<D, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Return the number of documents in the database.
    fn db_count<D: DocBuf>(&self, _doc: &D) -> Result<usize, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    /// Return the number of documents in the database given a predicate.
    fn db_count_where(
        &self,
        _vtable_id: &VTableId,
        _predicate: Self::Predicate,
    ) -> Result<usize, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }

    fn db_vtable_ids(&self) -> Result<&HashSet<VTableId>, Error> {
        unimplemented!("DocBufDbMngr method not implemented");
    }
}
