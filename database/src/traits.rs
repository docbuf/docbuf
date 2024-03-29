use crate::{partition_key::PartitionKey, Error};

use std::collections::HashSet;

use docbuf_core::{traits::*, vtable::VTableId};

/// DocBufDb is a trait used to interact with the DocBufDataBase for DocBuf documents.
pub trait DocBufDbMngr {
    /// The predicate type used for querying the database.
    type Predicate;

    /// Write a document into the database.
    /// This will return the document id.
    fn insert<D: DocBuf>(
        &self,
        doc: &D,
        partition_key: PartitionKey,
    ) -> Result<docbuf_core::uuid::Uuid, Error>;

    /// Return all documents in the database.
    fn all<D: DocBuf>(&self, doc: &D) -> Result<Vec<D>, Error>;

    /// Read documents in the database given a predicate.
    fn find<D: DocBuf>(&self, doc: &D, predicate: Self::Predicate) -> Result<Vec<D>, Error>;

    /// Get a document from the database.
    fn get<D: DocBuf>(&self, id: docbuf_core::uuid::Uuid) -> Result<D, Error>;

    /// Update a document in the database.
    fn update<D: DocBuf>(&self, doc: &D) -> Result<(), Error>;

    /// Delete a document from the database.
    fn delete<D: DocBuf>(&self, doc: D) -> Result<D, Error>;

    /// Return the number of documents in the database.
    fn count<D: DocBuf>(&self, doc: &D) -> Result<usize, Error>;

    /// Return the number of documents in the database given a predicate.
    fn count_where(&self, vtable_id: &VTableId, predicate: Self::Predicate)
        -> Result<usize, Error>;

    /// Return the vtable ids for the database.
    fn vtable_ids(&self) -> Result<&HashSet<VTableId>, Error>;
}

/// DocBufDb is a trait used to interact with the DocBufDataBase for DocBuf documents.
pub trait DocBufDb: DocBuf {
    /// The database client type.
    type Db: DocBufDbMngr;

    /// The predicate type used for querying the database.
    type Predicate;

    /// Return a database client.
    fn db() -> Result<Self::Db, Error>;

    /// Write a document into the database.
    /// This will return the document id.
    fn db_insert(&self) -> Result<docbuf_core::uuid::Uuid, Error>;

    /// Return all documents in the database.
    fn db_all() -> Result<Vec<Self::Doc>, Error>;

    /// Read documents in the database given a predicate.
    fn db_find(predicate: Self::Predicate) -> Result<Vec<Self::Doc>, Error>;

    /// Get a document from the database.
    fn db_get(
        id: docbuf_core::uuid::Uuid,
        partition_key: Option<impl Into<PartitionKey>>,
    ) -> Result<Self::Doc, Error>;

    /// Get all documents in a partition from the database.
    fn db_get_partition(partition_key: impl Into<PartitionKey>) -> Result<Vec<Self::Doc>, Error>;

    /// Update a document in the database.
    fn db_update(&self) -> Result<(), Error>;

    /// Delete a document from the database.
    fn db_delete(self) -> Result<Self::Doc, Error>;

    /// Delete a document partition from the database.
    fn db_delete_partition(partition_key: impl Into<PartitionKey>) -> Result<(), Error>;

    /// Return the number of documents in the database.
    fn db_count() -> Result<usize, Error>;

    /// Return the number of documents in the database given a predicate.
    fn db_count_where(predicate: Self::Predicate) -> Result<usize, Error>;

    /// Return the number of documents in the database given a partition key.
    fn db_count_partition(partition_key: impl Into<PartitionKey>) -> Result<usize, Error>;

    /// Return the database partition key for the document.
    /// This is used by the database to partition the document
    /// based on the user's requirements, grouping all of the documents
    /// with the same field value in the same partition.
    ///
    /// For example, if the document has a `customer_id` field,
    /// the partition key may be set to the customer id to quickly
    /// read/write customer_id records. This can also be used for
    /// bulk operations, where a user may wish to delete all records
    /// for a specific customer_id, whereby removing the need to search across
    /// multiple partitions, and only search for the specific partition containing the
    /// target documents.
    ///
    /// ```
    /// #[docbuf]
    /// struct MyStruct {
    ///  #[docbuf { partition_key = true }]
    ///  customer_id: String,
    /// }
    /// ```
    ///
    /// By default, this method uses the uuid as the partition key.
    fn partition_key(&self) -> Result<PartitionKey, Error> {
        Ok(PartitionKey::from(self.uuid()?))
    }
}
