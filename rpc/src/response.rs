use crate::StreamId;

use std::collections::HashMap;
use std::sync::mpsc::{channel, sync_channel, Receiver, SendError, Sender, SyncSender};

pub struct RpcResponse<Header> {
    pub(crate) stream_id: StreamId,
    pub(crate) headers: Vec<Header>,
    pub(crate) body: Vec<u8>,
}

impl<Header> RpcResponse<Header> {
    pub fn new(stream_id: StreamId, headers: Vec<Header>, body: Vec<u8>) -> Self {
        Self {
            stream_id,
            headers,
            body,
        }
    }

    // If the response is partially written, return a partial response, setting the bytes_written.
    pub fn consumed_partial(self, bytes_written: usize) -> RpcPartialResponse<Header> {
        RpcPartialResponse {
            stream_id: self.stream_id,
            headers: None,
            body: self.body,
            written: bytes_written,
        }
    }
}

pub struct RpcPartialResponse<Header> {
    pub(crate) stream_id: StreamId,
    pub(crate) headers: Option<Vec<Header>>,
    pub(crate) body: Vec<u8>,
    pub(crate) written: usize,
}

impl<Header> Into<RpcPartialResponse<Header>> for RpcResponse<Header> {
    fn into(self) -> RpcPartialResponse<Header> {
        RpcPartialResponse {
            stream_id: self.stream_id,
            headers: Some(self.headers),
            body: self.body,
            written: 0,
        }
    }
}

impl<Header> RpcPartialResponse<Header> {
    pub fn new(stream_id: StreamId) -> Self {
        Self {
            stream_id,
            headers: None,
            body: Vec::new(),
            written: 0,
        }
    }
}

pub struct RpcPartialResponses<T>(pub(crate) HashMap<StreamId, RpcPartialResponse<T>>);

impl<T> RpcPartialResponses<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, partial: RpcPartialResponse<T>) {
        self.0.insert(partial.stream_id, partial);
    }

    pub fn remove(&mut self, stream_id: StreamId) -> Option<RpcPartialResponse<T>> {
        self.0.remove(&stream_id)
    }
}

pub type RpcResponseSyncSender<Header> = SyncSender<RpcResponse<Header>>;
pub type RpcResponseReceiver<Header> = Receiver<RpcResponse<Header>>;

pub type RpcResponseSendError = SendError<RpcResponse<quiche::h3::Header>>;

impl RpcResponse<quiche::h3::Header> {
    /// Create a channel for sending and receiving requests.
    pub fn oneshot() -> (
        RpcResponseSyncSender<quiche::h3::Header>,
        RpcResponseReceiver<quiche::h3::Header>,
    ) {
        sync_channel(1)
    }
}
