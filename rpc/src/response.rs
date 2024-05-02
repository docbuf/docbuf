use crate::{RpcHeaders, StreamId};

use std::collections::HashMap;
use std::sync::mpsc::{channel, sync_channel, Receiver, SendError, Sender, SyncSender};

#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub stream_id: StreamId,
    pub headers: RpcHeaders,
    pub body: Vec<u8>,
}

impl RpcResponse {
    pub fn new(stream_id: StreamId, headers: RpcHeaders, body: Vec<u8>) -> Self {
        Self {
            stream_id,
            headers,
            body,
        }
    }

    pub fn with_empty_body(stream_id: StreamId, headers: impl Into<RpcHeaders>) -> Self {
        Self {
            stream_id,
            headers: headers.into(),
            body: Vec::new(),
        }
    }

    // If the response is partially written, return a partial response, setting the bytes_written.
    pub fn consumed_partial(self, bytes_written: usize) -> PartialRpcResponse {
        PartialRpcResponse {
            stream_id: self.stream_id,
            headers: None,
            body: self.body,
            written: bytes_written,
        }
    }
}

pub struct PartialRpcResponse {
    pub(crate) stream_id: StreamId,
    pub(crate) headers: Option<RpcHeaders>,
    pub(crate) body: Vec<u8>,
    pub(crate) written: usize,
}

impl Into<PartialRpcResponse> for RpcResponse {
    fn into(self) -> PartialRpcResponse {
        PartialRpcResponse {
            stream_id: self.stream_id,
            headers: Some(self.headers),
            body: self.body,
            written: 0,
        }
    }
}

impl Into<RpcResponse> for PartialRpcResponse {
    fn into(self) -> RpcResponse {
        RpcResponse {
            stream_id: self.stream_id,
            headers: self.headers.unwrap_or(RpcHeaders::empty()),
            body: self.body,
        }
    }
}

impl PartialRpcResponse {
    pub fn new(stream_id: StreamId, headers: impl Into<RpcHeaders>) -> Self {
        let headers = headers.into();

        let body = headers
            .content_length()
            .map(|content_length| vec![0; content_length])
            .unwrap_or_default();

        Self {
            stream_id,
            headers: Some(headers),
            body,
            written: 0,
        }
    }

    pub fn append_data(&mut self, data: &[u8]) {
        self.written += data.len();
        self.body.extend_from_slice(data);
    }
}

pub struct PartialRpcResponses(pub(crate) HashMap<StreamId, PartialRpcResponse>);

impl PartialRpcResponses {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, partial: PartialRpcResponse) {
        self.0.insert(partial.stream_id, partial);
    }

    pub fn remove(&mut self, stream_id: &StreamId) -> Option<PartialRpcResponse> {
        self.0.remove(stream_id)
    }

    pub fn get_mut(&mut self, stream_id: &StreamId) -> Option<&mut PartialRpcResponse> {
        self.0.get_mut(stream_id)
    }
}

pub type RpcResponseSyncSender = SyncSender<RpcResponse>;
pub type RpcResponseReceiver = Receiver<RpcResponse>;

pub type RpcResponseSendError = SendError<RpcResponse>;

pub struct RpcResponseSyncSenders(pub(crate) HashMap<StreamId, RpcResponseSyncSender>);

impl RpcResponseSyncSenders {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, stream_id: StreamId, sender: RpcResponseSyncSender) {
        self.0.insert(stream_id, sender);
    }

    pub fn remove(&mut self, stream_id: &StreamId) -> Option<RpcResponseSyncSender> {
        self.0.remove(stream_id)
    }

    pub fn get_mut(&mut self, stream_id: &StreamId) -> Option<&mut RpcResponseSyncSender> {
        self.0.get_mut(stream_id)
    }
}

impl RpcResponse {
    /// Create a `oneshot` channel for sending and receiving requests.
    pub fn oneshot() -> (RpcResponseSyncSender, RpcResponseReceiver) {
        sync_channel(1)
    }

    /// Returns a new map for response sync senders.
    pub fn sync_senders() -> RpcResponseSyncSenders {
        RpcResponseSyncSenders::new()
    }
}
