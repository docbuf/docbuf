use std::sync::{MutexGuard, TryLockError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] docbuf_core::error::Error),
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
    TransportError(#[from] crate::TransportErrorCode),
    #[error("TLS Error: {0}")]
    TlsError(String),
    #[error(transparent)]
    RpcRequestSendError(#[from] crate::RpcRequestSendError),
    #[error(transparent)]
    RpcResponseSendError(#[from] crate::RpcResponseSendError),
    #[error(transparent)]
    RecvError(#[from] std::sync::mpsc::RecvError),
    #[error(transparent)]
    TryRecvError(#[from] std::sync::mpsc::TryRecvError),
    #[error("Thread Join Error")]
    ThreadJoinError(Box<dyn std::any::Any + Send>),
    #[error("Invalid Header Value")]
    InvalidHeader,
    #[error("Missing Header Value: {0}")]
    MissingHeader(String),
    #[error("Method Not Found: {0}, {1}")]
    MethodNotFound(String, String),
    #[error("Failed to lock RPC context")]
    RpcContextLockError,
    #[error("Missing Request Body")]
    MissingRequestBody,
    #[error("Unknown Stream ID: {0}")]
    UnknownStreamId(u64),
    #[error("Response Timed Out")]
    ResponseTimedOut,
    #[error("Internal Error")]
    InternalError,
}
