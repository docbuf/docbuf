#[cfg(feature = "crypto")]
use crate::crypto::digest;

use nom::error::ErrorKind;
use thiserror::Error as ThisError;


#[derive(ThisError, Debug)]
pub enum Error {
    /// Missing Pragma
    #[error("No pragma found; expected 'docbuf v1;'")]
    MissingPragma,
    /// Invalid Path
    #[error("Invalid path")]
    InvalidPath,
    /// IO Error
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    /// Nom Error
    #[error("Nom Error: {0}")]
    Nom(nom::Err<(String, ErrorKind)>),
    /// Token Error
    #[error("Token Error: {0}")]
    Token(String),
    /// Invalid Module
    #[error("Invalid module. There must be one module name declared per file.")]
    InvalidModule,
    #[cfg(feature = "crypto")]
    /// Hash Digest Error
    /// #[cfg(feature = "crypto")]
    #[error(transparent)]
    #[cfg(feature = "crypto")]
    InvalidBufferSize(#[from] digest::InvalidBufferSize),
    /// Invalid Field Type
    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),
}

// Convert the &str error from nom to the owned Error type
impl From<nom::Err<(&str, ErrorKind)>> for Error {
    fn from(value: nom::Err<(&str, ErrorKind)>) -> Self {
        Error::Nom(value.to_owned())
    }
}

