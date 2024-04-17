use crate::{Partition, PartitionKey};

use super::Error;

use std::{fs::File, io::Read, path::PathBuf, str::FromStr};

use docbuf_core::vtable::{VTable, VTableId};
use serde::{Deserialize, Serialize};

/// Default directory for the DocBuf database.
/// This value is used if the directory is not specified in the configuration file.
pub const DEFAULT_DB_DIRECTORY: &str = "/tmp/.docbuf/db/";

// vtables/:vtable_id/pages/:page_id.dbp;

/// Options for the DocBufDbManager
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DocBufDbConfig {
    directory: Option<PathBuf>,
}

impl DocBufDbConfig {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path: PathBuf = path.into();

        println!("DocBuf DB Config Path: {:?}", path);

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
        partition_key: impl Into<PartitionKey>,
    ) -> Result<Partition, Error> {
        let id: VTableId = vtable_id.into();
        let vtable = VTable::from_file(self.vtable_file(&id)?)?;
        let key: PartitionKey = partition_key.into();
        let dir = self.vtable_directory(&id)?;
        let partition_path = dir.join(format!("{}.dbp", key.as_hex()));
        Ok(Partition::load(&vtable, partition_path)?)
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
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_docbuf_db_config() -> Result<(), Box<dyn std::error::Error>> {
        use super::*;

        let config = DocBufDbConfig::load("/tmp/.docbuf/db/config.toml")?;

        assert_eq!(config.directory()?, &PathBuf::from("/tmp/.docbuf/db"));

        Ok(())
    }
}
