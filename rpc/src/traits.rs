use crate::error::Error;

/// Remote Procedural Call (RPC) Server Trait for Implementing an RPC layer for
/// DocBuf enabled documents.
pub trait DocBufRpcServer {
    /// Configuration for the RPC server.
    type Config;

    /// Start the RPC server.
    fn start(&self) -> Result<(), Error>;

    /// Accept a new connection.
    fn accept(&self) -> Result<(), Error>;
}

/// Remote Procedural Call (RPC) Client Trait for Implementing an RPC layer for
/// DocBuf enabled documents.
pub trait DocBufRpcClient {}
