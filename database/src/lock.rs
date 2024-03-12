use crate::Error;

use std::fs::File;
use std::io::{Read, Write};
use std::{collections::HashSet, path::PathBuf};

use docbuf_core::vtable::VTableId;

use serde::{Deserialize, Serialize};

/// Default lock file for the DocBuf database.
pub const DEFAULT_LOCK_FILE: &str = "db.lock";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocBufDbLockState {
    pub vtables: HashSet<VTableId>,
}

#[derive(Debug, Clone, Default)]
pub struct DocBufDbLock {
    pub file: PathBuf,
    pub state: DocBufDbLockState,
}

impl DocBufDbLock {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        let file = directory.into().join(DEFAULT_LOCK_FILE);

        Self {
            file,
            state: DocBufDbLockState::default(),
        }
    }

    pub fn load(directory: impl Into<PathBuf>) -> Result<Self, Error> {
        let file = directory.into().join(DEFAULT_LOCK_FILE);
        let state = if file.exists() {
            let mut file = File::open(&file)?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;

            toml::from_str(buf.as_str())?
        } else {
            DocBufDbLockState::default()
        };

        Ok(Self { file, state })
    }

    /// Save state to toml lock file.
    pub fn save(&self) -> Result<(), Error> {
        let mut file = File::create(&self.file)?;
        let toml = toml::to_string(&self.state)?;
        file.write_all(toml.as_bytes())?;

        Ok(())
    }
}
