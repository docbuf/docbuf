use docbuf_core::traits::DocBuf;

use crate::{RpcHeader, RpcHeaders, RpcResponse, RpcResponseSyncSender, StreamId};

use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, SendError, Sender, SyncSender},
};

/// Partial RPC Requests are requests that whose body length is less than the expected
/// content length in the headers.
pub struct PartialRpcRequests(pub(crate) HashMap<StreamId, RpcRequest>);

impl PartialRpcRequests {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, request: RpcRequest) {
        self.0.insert(request.stream_id, request);
    }

    pub fn remove(&mut self, stream_id: &StreamId) -> Option<RpcRequest> {
        self.0.remove(stream_id)
    }

    pub fn get(&self, stream_id: &StreamId) -> Option<&RpcRequest> {
        self.0.get(stream_id)
    }

    pub fn get_mut(&mut self, stream_id: &StreamId) -> Option<&mut RpcRequest> {
        self.0.get_mut(stream_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&StreamId, &RpcRequest)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RpcRequest {
    pub stream_id: StreamId,
    pub headers: RpcHeaders,
    pub body: Option<Vec<u8>>,
}

impl RpcRequest {
    pub fn add_header(mut self, header: RpcHeader) -> Self {
        self.headers.inner.push(header);

        self
    }

    pub fn add_headers(mut self, headers: RpcHeaders) -> Self {
        self.headers.inner.extend(headers.inner);

        self
    }

    pub fn add_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);

        self
    }

    pub fn has_body(stream_id: StreamId, headers: RpcHeaders) -> Self {
        let body = headers.content_length().map(|len| vec![0; len]);

        Self {
            stream_id,
            headers,
            body,
        }
    }

    pub fn with_body(stream_id: StreamId, headers: RpcHeaders, body: Vec<u8>) -> Self {
        Self {
            stream_id,
            headers,
            body: Some(body),
        }
    }

    pub fn without_body(stream_id: StreamId, headers: RpcHeaders) -> Self {
        Self {
            stream_id,
            headers,
            body: None,
        }
    }

    pub fn as_docbuf<Doc: DocBuf>(&mut self) -> Result<<Doc as DocBuf>::Doc, crate::Error> {
        Ok(self
            .body
            .as_mut()
            .map(Doc::from_docbuf)
            .ok_or(crate::Error::MissingRequestBody)??)
    }

    pub fn body_size(&self) -> usize {
        self.body.as_ref().map_or(0, |body| body.len())
    }

    /// Return true if the content length is empty (i.e. bodyless)
    /// or if the body size is equal to the content length.
    pub fn is_complete(&self) -> bool {
        self.headers
            .content_length()
            .map_or(true, |len| self.body_size() == len)
    }
}

pub type RpcRequestSender = Sender<(RpcRequest, RpcResponseSyncSender)>;
pub type RpcRequestReceiver = Receiver<(RpcRequest, RpcResponseSyncSender)>;

pub type RpcRequestSendError = SendError<(RpcRequest, SyncSender<RpcResponse>)>;

impl RpcRequest {
    /// Create a channel for sending and receiving requests.
    pub fn channel() -> (RpcRequestSender, RpcRequestReceiver) {
        channel()
    }
}
