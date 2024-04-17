#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid Socket Address")]
    InvalidSocketAddress,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
    #[error(transparent)]
    Quiche(#[from] quiche::Error),
    #[error(transparent)]
    Http3(#[from] quiche::h3::Error),
    #[error("Invalid Connection Id")]
    InvalidConnectionId,
    #[error("Invalid Connection")]
    InvalidConnection,
    #[error("Failed HMAC Key Generation")]
    HmacKeyGeneration,
    #[error("Failed Connection ID Generation")]
    ConnectionIdGeneration,
    #[error("Connections Lock Poisoned")]
    ConnectionsLockPoisioned,
    #[error("Invalid Authentication Token")]
    InvalidAuthToken,
    #[error("HTTP/3 Connection Error")]
    Http3ConnectionNotFound,
    #[error("HTTP/3 Request Error")]
    Http3RequestError,
    #[error(transparent)]
    TransportError(#[from] crate::quic::TransportErrorCode),
    #[error("TLS Error: {0}")]
    TlsError(String),
}
