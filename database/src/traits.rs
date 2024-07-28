use crate::{Error, PartitionKey};

use docbuf_core::{traits::*, vtable::VTableId};

/// DocBufDb is a trait used to interact with the DocBufDataBase for DocBuf documents.
pub trait DocBufDbMngr {
    /// The predicate type used for querying the database.
    type Predicate;

    /// Write a document into the database.
    /// This will return the document id.
    fn insert<D: DocBuf>(&self, doc: &D, partition_key: PartitionKey) -> Result<D::DocId, Error>;

    /// Return all document IDs in the database for a specific DocBuf type. Optionally, provide a partition key
    /// to search a single partition. If not partition key is provided, returns for all partitions.
    fn all<D: DocBuf>(
        &self,
        partition_key: Option<PartitionKey>,
    ) -> Result<impl Iterator<Item = D::DocId>, Error>;

    /// Read documents in the database given a predicate.
    fn find<D: DocBuf>(
        &self,
        predicate: Self::Predicate,
        partition_key: Option<PartitionKey>,
    ) -> Result<impl Iterator<Item = D::Doc>, Error>;

    /// Get a document from the database.
    fn get<D: DocBuf>(
        &self,
        id: D::DocId,
        partition_key: Option<PartitionKey>,
    ) -> Result<Option<<D as DocBuf>::Doc>, Error>;

    /// Update a document in the database.
    fn update<D: DocBuf>(&self, doc: &D, partition_key: PartitionKey) -> Result<(), Error>;

    /// Delete a document from the database.
    fn delete<D: DocBuf>(&self, doc: D, partition_key: PartitionKey) -> Result<D::Doc, Error>;

    /// Return the number of documents in the database.
    fn count<D: DocBuf>(
        &self,
        predicate: Option<Self::Predicate>,
        partition_key: Option<PartitionKey>,
    ) -> Result<usize, Error>;

    /// Return the vtable ids for the database.
    fn vtable_ids(&self) -> Result<Vec<VTableId>, Error>;
}

/// DocBufDb is a trait used to interact with the DocBufDataBase for DocBuf documents.
pub trait DocBufDb: DocBuf {
    /// The database client type.
    type Db: DocBufDbMngr;

    /// The predicate type used for querying the database.
    type Predicate;

    // /// Return a database client.
    // fn db() -> Result<Self::Db, Error>;

    /// Write a document into the database.
    /// This will return the document id.
    fn db_insert(&self, db: &Self::Db) -> Result<Self::DocId, Error>;

    /// Return all documents in the database.
    /// If a partition key is provided, this method will
    /// return all documents in the corresponding partition.
    fn db_all(
        db: &Self::Db,
        partition_key: Option<impl Into<PartitionKey>>,
    ) -> Result<impl Iterator<Item = Self::DocId>, Error>;

    /// Read documents in the database given a predicate.
    fn db_find(
        db: &Self::Db,
        predicate: impl Into<Self::Predicate>,
        partition_key: Option<impl Into<PartitionKey>>,
    ) -> Result<impl Iterator<Item = Self::Doc>, Error>;

    /// Get a document from the database.
    fn db_get(
        db: &Self::Db,
        id: Self::DocId,
        partition_key: Option<impl Into<PartitionKey>>,
    ) -> Result<Option<Self::Doc>, Error>;

    /// Update a document in the database.
    fn db_update(&self, db: &Self::Db) -> Result<(), Error>;

    /// Delete a document from the database.
    fn db_delete(self, db: &Self::Db) -> Result<Self::Doc, Error>;

    /// Return the number of documents in the database.
    fn db_count(
        db: &Self::Db,
        partition_key: Option<impl Into<PartitionKey>>,
    ) -> Result<usize, Error>;

    /// Return the number of documents in the database given a predicate.
    fn db_count_where(
        db: &Self::Db,
        predicate: impl Into<Self::Predicate>,
        partition_key: Option<impl Into<PartitionKey>>,
    ) -> Result<usize, Error>;

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
        Ok(PartitionKey::from(self.uuid()?.into()))
    }
}
