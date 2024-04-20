use crate::{RpcResponse, RpcResponseSyncSender, StreamId};

use std::sync::mpsc::{channel, Receiver, SendError, Sender, SyncSender};

#[derive(Debug, Clone)]
pub struct RpcRequest<Header: Clone + std::fmt::Debug> {
    pub(crate) stream_id: StreamId,
    pub(crate) headers: Vec<Header>,
    pub(crate) body: Option<Vec<u8>>,
}

impl<Header> RpcRequest<Header>
where
    Header: Clone + std::fmt::Debug,
{
    pub fn with_body(stream_id: StreamId, headers: Vec<Header>, body: Vec<u8>) -> Self {
        Self {
            stream_id,
            headers,
            body: Some(body),
        }
    }

    pub fn without_body(stream_id: StreamId, headers: Vec<Header>) -> Self {
        Self {
            stream_id,
            headers,
            body: None,
        }
    }
}

pub type RpcRequestSender<Header> = Sender<(RpcRequest<Header>, RpcResponseSyncSender<Header>)>;
pub type RpcRequestReceiver<Header> = Receiver<(RpcRequest<Header>, RpcResponseSyncSender<Header>)>;

pub type RpcRequestSendError = SendError<(
    RpcRequest<quiche::h3::Header>,
    SyncSender<RpcResponse<quiche::h3::Header>>,
)>;

impl RpcRequest<quiche::h3::Header> {
    /// Create a channel for sending and receiving requests.
    pub fn channel() -> (
        RpcRequestSender<quiche::h3::Header>,
        RpcRequestReceiver<quiche::h3::Header>,
    ) {
        channel()
    }
}
