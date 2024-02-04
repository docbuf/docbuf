use nom::error::{ErrorKind, ParseError};
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
}

// Convert the &str error from nom to the owned Error type
impl From<nom::Err<(&str, ErrorKind)>> for Error {
    fn from(value: nom::Err<(&str, ErrorKind)>) -> Self {
        Error::Nom(value.to_owned())
    }
}
