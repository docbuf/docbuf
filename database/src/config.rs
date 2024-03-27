use super::*;

use std::{fs::File, io::Read, path::PathBuf, str::FromStr};

use docbuf_core::vtable::VTableId;
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

    pub fn vtable_lock_file(&self, vtable_id: &VTableId) -> Result<PathBuf, Error> {
        let dir = self.vtable_directory(vtable_id)?;
        Ok(dir.join(format!("{}.lock", vtable_id.as_hex())))
    }

    pub fn vtable_directory(&self, vtable_id: &VTableId) -> Result<PathBuf, Error> {
        let dir = self.vtables_directory()?;
        Ok(dir.join(vtable_id.as_hex()))
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
