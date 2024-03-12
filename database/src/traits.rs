use crate::Error;

use std::collections::HashSet;

use docbuf_core::{traits::*, vtable::VTableId};

/// DocBufDb is a trait used to interact with the DocBufDataBase for DocBuf documents.
pub trait DocBufDbMngr {
    /// The unique document id type.
    type DbDocId: AsRef<[u8]>;

    /// The predicate type used for querying the database.
    type Predicate;

    fn db_doc_id<D: DocBuf>(&self, doc: &D) -> Result<Option<Self::DbDocId>, Error>;

    /// Write a document into the database.
    /// This will return the document id.
    fn db_insert<D: DocBuf>(&self, doc: &D) -> Result<Self::DbDocId, Error>;

    /// Return all documents in the database.
    fn db_all<D: DocBuf>(&self, doc: &D) -> Result<Vec<D>, Error>;

    /// Read documents in the database given a predicate.
    fn db_find<D: DocBuf>(&self, doc: &D, predicate: Self::Predicate) -> Result<Vec<D>, Error>;

    /// Get a document from the database.
    fn db_get<D: DocBuf>(&self, id: Self::DbDocId) -> Result<D, Error>;

    /// Update a document in the database.
    fn db_update<D: DocBuf>(&self, doc: &D) -> Result<(), Error>;

    /// Delete a document from the database.
    fn db_delete<D: DocBuf>(&self, doc: D) -> Result<D, Error>;

    /// Return the number of documents in the database.
    fn db_count<D: DocBuf>(&self, doc: &D) -> Result<usize, Error>;

    /// Return the number of documents in the database given a predicate.
    fn db_count_where(
        &self,
        vtable_id: &VTableId,
        predicate: Self::Predicate,
    ) -> Result<usize, Error>;

    /// Return the vtable ids for the database.
    fn db_vtable_ids(&self) -> Result<&HashSet<VTableId>, Error>;
}

/// DocBufDb is a trait used to interact with the DocBufDataBase for DocBuf documents.
pub trait DocBufDb: DocBuf {
    /// The database client type.
    type Db: DocBufDbMngr;

    /// The unique document id type.
    type DbDocId: AsRef<[u8]>;

    /// The predicate type used for querying the database.
    type Predicate;

    /// Return a database client.
    fn db() -> Result<Self::Db, Error>;

    fn db_doc_id(&self) -> Result<Option<Self::DbDocId>, Error>;

    /// Write a document into the database.
    /// This will return the document id.
    fn db_insert(&self) -> Result<Self::DbDocId, Error>;

    /// Return all documents in the database.
    fn db_all() -> Result<Vec<Self::Doc>, Error>;

    /// Read documents in the database given a predicate.
    fn db_find(predicate: Self::Predicate) -> Result<Vec<Self::Doc>, Error>;

    /// Get a document from the database.
    fn db_get(id: Self::DbDocId) -> Result<Self::Doc, Error>;

    /// Update a document in the database.
    fn db_update(&self) -> Result<(), Error>;

    /// Delete a document from the database.
    fn db_delete(self) -> Result<Self::Doc, Error>;

    /// Return the number of documents in the database.
    fn db_count() -> Result<usize, Error>;

    /// Return the number of documents in the database given a predicate.
    fn db_count_where(vtable_id: &VTableId, predicate: Self::Predicate) -> Result<usize, Error>;
}
