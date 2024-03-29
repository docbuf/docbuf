pub mod config;
pub mod lock;
pub mod manager;
pub mod page;
pub mod partition_key;
pub mod traits;

pub use config::*;
pub use lock::*;
pub use manager::*;
pub use page::*;
pub use partition_key::*;
pub use traits::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DocBuf(#[from] docbuf_core::error::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    #[error("VTable ID not found")]
    VTableIdNotFound,
    #[error("Database directory not set")]
    DirectoryNotSet,
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error("VTable Page Error: {0}")]
    PageError(String),
    #[error("VTable Page Lock Error: {0}")]
    PageLockError(String),
    #[error("Max pages reached for vtable: {0}")]
    MaxPagesReached(VTableIdAsHex),
}
