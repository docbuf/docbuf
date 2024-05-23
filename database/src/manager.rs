use super::*;

use crate::DocBufDbMngr;
use crate::Error;

use docbuf_core::traits::DocBuf;
use docbuf_core::vtable::*;

use std::collections::HashSet;
use std::io::ErrorKind;
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

    pub fn save_config(&self, config_path: impl Into<PathBuf>) -> Result<(), Error> {
        let config_path = config_path.into();
        self.config.save(config_path)?;

        Ok(())
    }

    pub fn write_docbuf(
        &self,
        vtable_id: &[u8; 8],
        partition: u16,
        offsets: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), Error> {
        self.config
            .partition_file(vtable_id, partition.into(), PartitionPermission::Write)?
            .write_docbuf(offsets, buffer)?;

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

    pub fn read_docbuf(
        &self,
        vtable_id: &[u8; 8],
        doc_id: &[u8; 16],
        partition_key: Option<[u8; 16]>,
    ) -> Result<Option<Vec<u8>>, Error> {
        let partition_id = PartitionId::from(partition_key.unwrap_or_else(|| *doc_id));

        match self
            .config
            .partition_file(vtable_id, partition_id, PartitionPermission::Read)
            .and_then(|mut partition| partition.read_docbuf(&doc_id))
        {
            Err(_) => Ok(None),
            Ok(docbuf) => Ok(docbuf),
        }
    }

    /// Delete the docbuf in the database.
    pub fn delete_docbuf(
        &self,
        vtable_id: &[u8; 8],
        doc_id: &[u8; 16],
        partition: u16,
    ) -> Result<Vec<u8>, Error> {
        self.config
            .partition_file(vtable_id, partition.into(), PartitionPermission::Update)?
            .delete_docbuf(doc_id, self.config.tombstone())
    }

    /// Migrate a docbuf from one partition to another.
    pub fn migrate_docbuf(
        &self,
        vtable_id: &[u8; 8],
        doc_id: &[u8; 16],
        partition_id: u16,
        offsets: Vec<u8>,
        buffer: Option<Vec<u8>>,
    ) -> Result<(), Error> {
        // Check if the doc_id exists in another partition.
        for partition in self
            .partitions(vtable_id, None, PartitionPermission::Update)?
            .iter_mut()
        {
            // If the doc_id exists in another partition, delete it and write it to the new partition.
            if partition.read_docbuf(&doc_id)?.is_some() && *partition.id() != partition_id {
                let docbuf = self.delete_docbuf(vtable_id, doc_id, *partition.id())?;

                // If there is a new buffer to write, use that, otherwise use the old buffer
                let docbuf = buffer.unwrap_or(docbuf);

                self.write_docbuf(vtable_id, partition_id, offsets, docbuf)?;

                return Ok(());
            }
        }

        // If the doc_id does not exist in any partition, create a new docbuf if a buffer is provided.
        if let Some(docbuf) = buffer {
            self.write_docbuf(vtable_id, partition_id, offsets, docbuf)?;
        }

        Ok(())
    }

    /// Update the docbuf in the database.
    pub fn update_docbuf(
        &self,
        vtable_id: &[u8; 8],
        doc_id: &[u8; 16],
        partition_id: u16,
        offsets: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), Error> {
        match self.config.partition_file(
            vtable_id,
            partition_id.into(),
            PartitionPermission::Update,
        ) {
            Ok(mut partition) => match partition.update_docbuf(doc_id, &offsets, &buffer) {
                Ok(_) => Ok(()),
                Err(Error::DocBufNotFound) => {
                    println!("DocBuf not found, attempting to migrate...");
                    self.migrate_docbuf(vtable_id, doc_id, partition_id, offsets, Some(buffer))?;

                    Ok(())
                }
                Err(e) => {
                    return Err(e);
                }
            },
            Err(e) => match e {
                Error::Io(e) => match e.kind() {
                    ErrorKind::NotFound => {
                        println!("Partition not found, attempting to migrate...");
                        self.migrate_docbuf(
                            vtable_id,
                            doc_id,
                            partition_id,
                            offsets,
                            Some(buffer),
                        )?;
                        Ok(())
                    }
                    _ => {
                        return Err(Error::from(e));
                    }
                },
                _ => {
                    return Err(e);
                }
            },
        }
    }

    fn partitions(
        &self,
        vtable_id: &[u8; 8],
        partition_id: Option<u16>,
        permission: PartitionPermission,
    ) -> Result<Vec<Partition>, Error> {
        Ok(match partition_id {
            Some(partition_id) => {
                vec![self
                    .config
                    .partition_file(vtable_id, partition_id.into(), permission)?]
            }
            None => self.config.vtable_partitions(vtable_id)?,
        })
    }

    /// Search for docbufs in the database that match the given predicate.
    /// The predicate is a statement that is evaluated for each docbuf in the
    /// database. If the predicate returns true, the docbuf is included in the
    /// result.
    pub fn search_docbufs(
        &self,
        vtable_id: &[u8; 8],
        partition_id: Option<u16>,
        predicates: Predicates,
    ) -> Result<impl Iterator<Item = Vec<u8>>, Error> {
        self.partitions(vtable_id, partition_id, PartitionPermission::Read)?
            .iter_mut()
            .map(|partition| partition.search_docbufs(&predicates))
            .collect::<Result<Vec<_>, _>>()
            .map(|docbufs| docbufs.into_iter().flatten())
    }

    pub fn read_docbuf_ids(
        &self,
        vtable_id: &[u8; 8],
        partition_id: Option<u16>,
    ) -> Result<impl Iterator<Item = [u8; 16]>, Error> {
        self.partitions(vtable_id, partition_id, PartitionPermission::Read)?
            .iter_mut()
            .map(|partition| partition.read_docbuf_ids())
            .collect::<Result<Vec<_>, _>>()
            .map(|ids| ids.into_iter().flatten())
    }

    /// Returns the total count of docbufs according to the vtable id across
    /// all partitions.
    pub fn docbuf_count(
        &self,
        vtable_id: &[u8; 8],
        predicate: Option<Predicates>,
        partition_id: Option<u16>,
    ) -> Result<usize, Error> {
        self.partitions(vtable_id, partition_id, PartitionPermission::Read)?
            .iter_mut()
            .try_fold(0, |acc, partition| {
                let count = partition.count(predicate.clone())?;
                Ok(acc + count)
            })
    }
}

impl DocBufDbMngr for DocBufDbManager {
    /// The predicate type used for querying the database.
    type Predicate = Predicates;

    /// Write a document into the database.
    /// This will return the document id.
    fn insert<D: DocBuf>(&self, doc: &D, partition_key: PartitionKey) -> Result<D::DocId, Error> {
        // Call the document UUID method to get the document id.
        // If the method fails, do not save the document.
        let id = doc.uuid()?;

        // Return the vtable for the document.
        let vtable = D::vtable()?;

        // Allocate a buffer for the document.
        let mut buffer = vtable.alloc_buf();

        let offsets = doc.to_docbuf(&mut buffer)?;

        // Write the document buffer to the database.
        self.write_docbuf(
            &vtable.id(),
            partition_key.bucket(None),
            offsets.to_vec(),
            buffer,
        )?;

        Ok(id)
    }

    /// Return all documents in the database.
    fn all<D: DocBuf>(
        &self,
        partition_key: Option<PartitionKey>,
    ) -> Result<impl Iterator<Item = D::DocId>, Error> {
        // Return the vtable for the document.
        let vtable_id = D::vtable()?.id().deref();
        let partition_id = partition_key.map(PartitionId::from).map(u16::from);

        let iter = self
            .read_docbuf_ids(vtable_id, partition_id)?
            .map(D::DocId::from);

        Ok(iter)
    }

    /// Read documents in the database given a predicate.
    fn find<D: DocBuf>(
        &self,
        predicate: Self::Predicate,
        partition_key: Option<PartitionKey>,
    ) -> Result<impl Iterator<Item = D::Doc>, Error> {
        let vtable_id = D::vtable()?.id().deref();
        let partition_id = partition_key.clone().map(PartitionId::from).map(u16::from);

        let iter = self
            .search_docbufs(vtable_id, partition_id, predicate)?
            .map(|mut buf| D::from_docbuf(&mut buf))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter();

        Ok(iter)
    }

    /// Get a document from the database.
    fn get<D: DocBuf>(
        &self,
        id: D::DocId,
        partition_key: Option<PartitionKey>,
    ) -> Result<Option<<D as DocBuf>::Doc>, Error> {
        let vtable_id = D::vtable()?.id().deref();

        self.read_docbuf(vtable_id, &id.into(), partition_key.map(|pk| *pk.deref()))?
            .map(|mut buf| D::from_docbuf(&mut buf).map_err(Error::from))
            .transpose()
    }

    /// Update a document in the database.
    fn update<D: DocBuf>(&self, doc: &D, partition_key: PartitionKey) -> Result<(), Error> {
        println!("Updating Document in Database");
        let vtable_id = D::vtable()?.id().deref();
        let mut buffer = D::vtable()?.alloc_buf();
        let offsets = doc.to_docbuf(&mut buffer)?;
        let doc_id = doc.uuid()?;

        self.update_docbuf(
            vtable_id,
            &doc_id.into(),
            partition_key.bucket(None),
            offsets.to_vec(),
            buffer,
        )
    }

    /// Delete a document from the database.
    fn delete<D: DocBuf>(&self, doc: D, partition_key: PartitionKey) -> Result<D::Doc, Error> {
        println!("Deleting Document in Database");
        let vtable_id = D::vtable()?.id().deref();
        let doc_id = doc.uuid()?;
        let partition = partition_key.bucket(None);
        let mut docbuf = self.delete_docbuf(vtable_id, &doc_id.into(), partition)?;
        Ok(D::from_docbuf(&mut docbuf)?)
    }

    /// Return the number of documents in the database.
    fn count<D: DocBuf>(
        &self,
        predicate: Option<Predicates>,
        partition_key: Option<PartitionKey>,
    ) -> Result<usize, Error> {
        let vtable_id = D::vtable()?.id().deref();
        let partition_id = partition_key.map(PartitionId::from).map(u16::from);

        self.docbuf_count(vtable_id, predicate, partition_id)
    }

    fn vtable_ids(&self) -> Result<Vec<VTableId>, Error> {
        self.config.vtable_ids()
    }
}
