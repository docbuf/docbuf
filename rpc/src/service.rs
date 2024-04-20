use crate::Error;

use docbuf_core::traits::DocBuf;

pub type Future<Res> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<Res, Error>> + Send>>;

pub type Iter<Res> = std::pin::Pin<Box<dyn std::iter::Iterator<Item = Result<Res, Error>> + Send>>;

pub enum RpcResult<Res> {
    Future(Future<Res>),
    Stream(Iter<Res>),
}

pub type RpcServiceMethodHandler<Req, Res> = Box<dyn Fn(Req) -> RpcResult<Res>>;

pub struct RpcServiceMethod<Req, Res>
where
    Req: DocBuf<Doc = Req>,
    Res: DocBuf<Doc = Res>,
{
    /// The name of the service that this method belongs to.
    service_name: String,
    /// The name of the method.
    method_name: String,
    /// The method handler.
    handler: RpcServiceMethodHandler<Req, Res>,
}

impl<Req, Res> RpcServiceMethod<Req, Res>
where
    Req: DocBuf<Doc = Req>,
    Res: DocBuf<Doc = Res>,
{
    pub fn new(
        service_name: impl Into<String>,
        method_name: impl Into<String>,
        handler: RpcServiceMethodHandler<Req, Res>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            method_name: method_name.into(),
            handler,
        }
    }
}
