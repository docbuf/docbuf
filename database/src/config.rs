use crate::{Partition, PartitionId, PartitionPermission};

use super::Error;

use std::{
    fs::File,
    io::{Read, Write},
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
};

use docbuf_core::vtable::{VTable, VTableId};
use serde::{Deserialize, Serialize};

/// Default directory for the DocBuf database.
/// This value is used if the directory is not specified in the configuration file.
pub const DEFAULT_DB_DIRECTORY: &str = "/tmp/.docbuf/db/";

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DocBufDbRpcConfig {
    pub server: Option<SocketAddr>,
    pub priv_key: Option<PathBuf>,
    pub cert_chain: Option<PathBuf>,
    pub root_cert: Option<PathBuf>,
}

/// Options for the DocBufDbManager
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DocBufDbConfig {
    pub directory: Option<PathBuf>,
    pub tombstone: Option<bool>,
    pub rpc: Option<DocBufDbRpcConfig>,
}

impl DocBufDbConfig {
    /// Set the RPC configuration for the database.
    pub fn set_rpc(mut self, rpc: DocBufDbRpcConfig) -> Self {
        self.rpc = Some(rpc);

        self
    }

    /// Returns the RPC configuration for the database.
    pub fn rpc(&self) -> Option<&DocBufDbRpcConfig> {
        self.rpc.as_ref()
    }

    /// Set the tombstone option for the database.
    /// If set to true, deleted docbuf records will be tombstoned,
    /// which will zero out the data, instead of removing the record from the database.
    pub fn set_tombstone(mut self, tombstone: bool) -> Self {
        self.tombstone = Some(tombstone);

        self
    }

    /// Returns whether the deleted docbuf record should be removed or tombstoned.
    /// By default, records are not tombstoned.
    pub fn tombstone(&self) -> bool {
        self.tombstone.unwrap_or(false)
    }

    pub fn load(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path: PathBuf = path.into();

        let mut file = File::open(&path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        // Deserialize the configuration file.
        let mut config: Self = toml::from_str(buf.as_str())?;

        // Setup directory with required folders.
        config.setup_directory()?;

        Ok(config)
    }

    pub fn vtable_lock_file(&self, vtable_id: impl Into<VTableId>) -> Result<PathBuf, Error> {
        let id: VTableId = vtable_id.into();
        let dir = self.vtable_directory(&id)?;
        Ok(dir.join(format!("{}.lock", id.as_hex())))
    }

    pub fn partition_file(
        &self,
        vtable_id: impl Into<VTableId>,
        partition_id: PartitionId,
        permission: PartitionPermission,
    ) -> Result<Partition, Error> {
        let id: VTableId = vtable_id.into();
        let vtable = VTable::from_file(self.vtable_file(&id)?)?;
        let dir = self.vtable_directory(&id)?;
        let partition_path = dir.join(format!("{}.dbp", partition_id.as_hex()));
        Ok(Partition::load(
            &vtable,
            partition_path,
            partition_id,
            permission,
        )?)
    }

    pub fn vtable_partitions(
        &self,
        vtable_id: impl Into<VTableId>,
    ) -> Result<Vec<Partition>, Error> {
        let id = vtable_id.into();
        self.vtable_directory(id.clone())?
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter_map(
                |entry| match entry.path().extension().and_then(|ext| ext.to_str()) {
                    ext if Some("dbp") == ext => entry
                        .path()
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|name| {
                            u16::from_str_radix(&name.replace("0x", ""), 16).map(PartitionId::from)
                        })
                        .transpose()
                        .unwrap_or(None),
                    _ => None,
                },
            )
            .map(|partition_id| {
                self.partition_file(id.clone(), partition_id, PartitionPermission::Read)
            })
            .collect()
    }

    pub fn vtable_directory(&self, vtable_id: impl Into<VTableId>) -> Result<PathBuf, Error> {
        let dir = self.vtables_directory()?;
        let id: VTableId = vtable_id.into();
        Ok(dir.join(id.as_hex()))
    }

    pub fn vtables_directory(&self) -> Result<PathBuf, Error> {
        let dir = self.directory()?;
        Ok(dir.join("vtables"))
    }

    /// Root DocBuf database directory
    pub fn directory(&self) -> Result<&PathBuf, Error> {
        self.directory.as_ref().ok_or(Error::DirectoryNotSet)
    }

    pub fn set_directory(mut self, directory: PathBuf) -> Self {
        self.directory = Some(directory);

        self
    }

    pub fn setup_directory(&mut self) -> Result<(), Error> {
        let dir = self
            .directory
            .clone()
            .unwrap_or(PathBuf::from_str(DEFAULT_DB_DIRECTORY)?);

        // Add the vtables directory
        let vtables = dir.join("vtables");

        if !vtables.exists() {
            std::fs::create_dir_all(&vtables)?;
        }

        self.directory = Some(dir);

        Ok(())
    }

    pub fn vtable_dir(&self) -> Result<PathBuf, Error> {
        let dir = self.directory()?;
        Ok(dir.join("vtables"))
    }

    pub fn vtable_file(&self, vtable_id: impl Into<VTableId>) -> Result<PathBuf, Error> {
        let id: VTableId = vtable_id.into();
        let dir = self.vtable_directory(&id)?;
        let vtable_path = dir.join(format!("{}.vtable", id.as_hex()));
        Ok(vtable_path)
    }

    pub fn write_vtable(&self, vtable: &VTable) -> Result<(), Error> {
        // Create the vtable folder.
        std::fs::create_dir_all(self.vtable_directory(vtable)?)?;

        let vtable_path = self.vtable_file(vtable.id())?;

        // Check if the vtable file exists.
        if vtable_path.exists() {
            // return early if the file exists.
            return Ok(());
        }

        Ok(vtable.to_file(vtable_path)?)
    }

    /// Read the vtable from the file system.
    pub fn read_vtable(&self, vtable_id: impl Into<VTableId>) -> Result<VTable, Error> {
        let id: VTableId = vtable_id.into();
        Ok(VTable::from_file(self.vtable_file(id)?)?)
    }

    /// List all the VTable IDs in the database.
    pub fn vtable_ids(&self) -> Result<Vec<VTableId>, Error> {
        let ids = self
            .vtable_dir()?
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|name| VTableId::from_hex(name).ok())
                    .unwrap_or(None)
            })
            .collect();

        Ok(ids)
    }

    /// Save the configuration to the file system.
    pub fn save(&self, path: impl Into<PathBuf>) -> Result<(), Error> {
        let path: PathBuf = path.into();
        let mut file = File::create(&path)?;
        file.write_all(toml::to_string(self)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_docbuf_db_config() -> Result<(), Box<dyn std::error::Error>> {
        use super::*;

        let config = DocBufDbConfig::default().set_directory("/tmp/.docbuf/db".into());

        assert_eq!(config.directory()?, &PathBuf::from("/tmp/.docbuf/db"));

        Ok(())
    }
}
