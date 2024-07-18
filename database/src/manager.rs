use super::*;

use crate::DocBufDbMngr;
use crate::Error;

use docbuf_core::traits::DocBuf;
use docbuf_core::vtable::*;
use docbuf_rpc::RpcClient;
use tracing::debug;

use std::io::ErrorKind;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DocBufDbManager {
    /// Configuration for the database
    pub config: DocBufDbConfig,
    /// Optional RPC Client
    pub rpc_client: Option<RpcClient>,
}

impl DocBufDbManager {
    pub fn from_config(config_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let config = DocBufDbConfig::load(config_path.into())?;

        let rpc_client = config
            .rpc()
            .and_then(|rpc| rpc.server)
            .map(|s| RpcClient::connect(s, None, None).ok())
            .flatten();

        Ok(Self { config, rpc_client })
    }

    pub fn connect(&mut self) -> Result<(), Error> {
        self.rpc_client = self
            .config
            .rpc()
            .and_then(|rpc| rpc.server)
            .map(|s| RpcClient::connect(s, None, None).ok())
            .ok_or_else(|| {
                Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    "Failed to connect to RPC",
                ))
            })?;

        Ok(())
    }

    pub fn save_config(&self, config_path: impl Into<PathBuf>) -> Result<(), Error> {
        let config_path = config_path.into();
        self.config.save(config_path)?;

        Ok(())
    }

    pub fn write_docbuf(&self, request: WriteDocBufRequest) -> Result<[u8; 16], Error> {
        self.config
            .partition_file(
                request.vtable_id,
                request.partition_id.into(),
                PartitionPermission::Write,
            )?
            .write_docbuf(request.offsets, request.buffer)
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

    pub fn read_docbuf(&self, request: ReadDocBufRequest) -> Result<Option<Vec<u8>>, Error> {
        match self
            .config
            .partition_file(
                request.vtable_id,
                request.partition_id.into(),
                PartitionPermission::Read,
            )
            .and_then(|mut partition| partition.read_docbuf(&request.doc_id))
        {
            Err(_) => Ok(None),
            Ok(docbuf) => Ok(docbuf),
        }
    }

    /// Delete the docbuf in the database.
    pub fn delete_docbuf(
        &self,
        request: DeleteDocBufRequest,
        // vtable_id: &[u8; 8],
        // doc_id: &[u8; 16],
        // partition: u16,
    ) -> Result<Vec<u8>, Error> {
        self.config
            .partition_file(
                &request.vtable_id,
                request.partition_id.into(),
                PartitionPermission::Update,
            )?
            .delete_docbuf(&request.doc_id, self.config.tombstone())
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
                let docbuf = self.delete_docbuf(DeleteDocBufRequest {
                    vtable_id: *vtable_id,
                    doc_id: *doc_id,
                    partition_id: *partition.id(),
                })?;

                // If there is a new buffer to write, use that, otherwise use the old buffer
                let docbuf = buffer.unwrap_or(docbuf);

                self.write_docbuf(WriteDocBufRequest {
                    vtable_id: vtable_id.to_owned(),
                    partition_id,
                    offsets,
                    buffer: docbuf,
                })?;

                return Ok(());
            }
        }

        // If the doc_id does not exist in any partition, create a new docbuf if a buffer is provided.
        if let Some(docbuf) = buffer {
            self.write_docbuf(WriteDocBufRequest {
                vtable_id: vtable_id.to_owned(),
                partition_id,
                offsets,
                buffer: docbuf,
            })?;
        }

        Ok(())
    }

    /// Update the docbuf in the database.
    pub fn update_docbuf(
        &self,
        request: UpdateDocBufRequest,
        // vtable_id: &[u8; 8],
        // doc_id: &[u8; 16],
        // partition_id: u16,
        // offsets: Vec<u8>,
        // buffer: Vec<u8>,
    ) -> Result<(), Error> {
        match self.config.partition_file(
            request.vtable_id,
            request.partition_id.into(),
            PartitionPermission::Update,
        ) {
            Ok(mut partition) => {
                match partition.update_docbuf(&request.doc_id, &request.offsets, &request.buffer) {
                    Ok(_) => Ok(()),
                    Err(Error::DocBufNotFound) => {
                        debug!("DocBuf not found, attempting to migrate...");
                        self.migrate_docbuf(
                            &request.vtable_id,
                            &request.doc_id,
                            request.partition_id,
                            request.offsets,
                            Some(request.buffer),
                        )?;

                        Ok(())
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => match e {
                Error::Io(e) => match e.kind() {
                    ErrorKind::NotFound => {
                        debug!("Partition not found, attempting to migrate...");
                        self.migrate_docbuf(
                            &request.vtable_id,
                            &request.doc_id,
                            request.partition_id,
                            request.offsets,
                            Some(request.buffer),
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
        request: CountDocBufRequest,
        // vtable_id: &[u8; 8],
        // predicate: Option<Predicates>,
        // partition_id: Option<u16>,
    ) -> Result<usize, Error> {
        self.partitions(
            &request.vtable_id,
            request.partition_id,
            PartitionPermission::Read,
        )?
        .iter_mut()
        .try_fold(0, |acc, partition| {
            let count = partition.count(request.predicate.clone())?;
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

        let request = WriteDocBufRequest {
            vtable_id: *vtable.id().to_owned(),
            partition_id: partition_key.bucket(None),
            offsets: offsets.to_vec(),
            buffer: buffer.to_vec(),
        };

        if let Some(rpc) = &self.rpc_client {
            Self::write_docbuf_rpc(rpc, request)?;
        } else {
            // Write the document buffer to the database.
            self.write_docbuf(request)?;
        };

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

        let doc_id: [u8; 16] = id.into();

        let partition_id: PartitionId = partition_key
            .unwrap_or_else(|| PartitionKey::from(doc_id.clone()))
            .into();

        let request = ReadDocBufRequest {
            vtable_id: *vtable_id,
            partition_id: partition_id.into(),
            doc_id,
        };

        if let Some(rpc) = &self.rpc_client {
            Self::read_docbuf_rpc(rpc, request)?.doc
        } else {
            // Write the document buffer to the database.
            self.read_docbuf(request)?
        }
        .map(|mut buf| D::from_docbuf(&mut buf).map_err(Error::from))
        .transpose()
    }

    /// Update a document in the database.
    fn update<D: DocBuf>(&self, doc: &D, partition_key: PartitionKey) -> Result<(), Error> {
        debug!("Updating Document in Database");
        let vtable_id = D::vtable()?.id().deref();
        let mut buffer = D::vtable()?.alloc_buf();
        let offsets = doc.to_docbuf(&mut buffer)?;
        let doc_id = doc.uuid()?;

        let request = UpdateDocBufRequest {
            vtable_id: *vtable_id,
            partition_id: partition_key.bucket(None),
            doc_id: doc_id.into(),
            offsets: offsets.to_vec(),
            buffer: buffer.to_vec(),
        };

        if let Some(rpc) = &self.rpc_client {
            Self::update_docbuf_rpc(rpc, request)?;
            Ok(())
        } else {
            self.update_docbuf(request)
        }
    }

    /// Delete a document from the database.
    fn delete<D: DocBuf>(&self, doc: D, partition_key: PartitionKey) -> Result<D::Doc, Error> {
        debug!("Deleting Document in Database");
        let vtable_id = D::vtable()?.id().deref();
        let doc_id = doc.uuid()?;
        let partition = partition_key.bucket(None);

        let request = DeleteDocBufRequest {
            vtable_id: *vtable_id,
            partition_id: partition,
            doc_id: doc_id.into(),
        };

        let mut docbuf = if let Some(rpc) = &self.rpc_client {
            Self::delete_docbuf_rpc(rpc, request)?.doc
        } else {
            self.delete_docbuf(request)?
        };

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

        let request = CountDocBufRequest {
            vtable_id: *vtable_id,
            partition_id,
            predicate,
        };

        if let Some(rpc) = &self.rpc_client {
            Ok(Self::count_docbuf_rpc(rpc, request)?.count)
        } else {
            self.docbuf_count(request)
        }
    }

    fn vtable_ids(&self) -> Result<Vec<VTableId>, Error> {
        self.config.vtable_ids()
    }
}
