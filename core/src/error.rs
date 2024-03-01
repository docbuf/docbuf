#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "std")]
    /// IO Error Type
    #[error(transparent)]
    IO(#[from] std::io::Error),
    // #[error(transparent)]
    // IDL(#[from] crate::idl::error::Error),
    /// Serde Error Type
    #[error("Error: {0}")]
    Serde(String),
    #[cfg(feature = "ed25519")]
    #[error(transparent)]
    #[cfg(feature = "ed25519")]
    Ed25519Signature(#[from] ed25519::signature::Error),
    #[error(transparent)]
    VTable(#[from] crate::vtable::Error),
    /// UTF-8 Error
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Failed to create initial buffer allocation")]
    FailedToCreateBuffer,
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}
