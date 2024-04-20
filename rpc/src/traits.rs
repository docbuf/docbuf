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

/// DocBufRpcService Trait for Implementing an RPC service for DocBuf enabled
/// documents.
pub trait DocBufRpcServiceMethods {
    type RpcServiceMethods;

    /// Returns the name of the service.
    fn service() -> &'static str;

    /// Returns the methods for the service.
    fn method(&self, method_name: &str) -> Self::RpcServiceMethods;
}
