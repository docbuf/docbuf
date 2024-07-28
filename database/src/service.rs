use super::*;

use std::sync::{Arc, Mutex};

use docbuf_core::serde::serde_bytes;
use docbuf_core::traits::DocBuf;
use docbuf_macros::*;
use serde::{Deserialize, Serialize};
use tracing::info;

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteDocBufRequest {
    #[serde(with = "serde_bytes")]
    pub vtable_id: [u8; 8],
    pub partition_id: u16,
    #[serde(with = "serde_bytes")]
    pub offsets: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub buffer: Vec<u8>,
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteDocBufResponse {
    #[serde(with = "serde_bytes")]
    pub doc_id: [u8; 16],
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDocBufRequest {
    #[serde(with = "serde_bytes")]
    pub vtable_id: [u8; 8],
    #[serde(with = "serde_bytes")]
    pub doc_id: [u8; 16],
    pub partition_id: u16,
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDocBufResponse {
    #[serde(with = "serde_bytes")]
    pub doc: Option<Vec<u8>>,
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocBufRequest {
    #[serde(with = "serde_bytes")]
    pub vtable_id: [u8; 8],
    #[serde(with = "serde_bytes")]
    pub doc_id: [u8; 16],
    pub partition_id: u16,
    #[serde(with = "serde_bytes")]
    pub offsets: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub buffer: Vec<u8>,
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocBufResponse {}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDocBufRequest {
    #[serde(with = "serde_bytes")]
    pub vtable_id: [u8; 8],
    #[serde(with = "serde_bytes")]
    pub doc_id: [u8; 16],
    pub partition_id: u16,
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDocBufResponse {
    #[serde(with = "serde_bytes")]
    pub doc: Vec<u8>,
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountDocBufRequest {
    #[serde(with = "serde_bytes")]
    pub vtable_id: [u8; 8],
    pub predicate: Option<Predicates>,
    pub partition_id: Option<u16>,
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountDocBufResponse {
    pub count: usize,
}

#[docbuf_rpc]
impl DocBufDbManager {
    fn write_docbuf_rpc(
        ctx: Arc<Mutex<Self>>,
        request: WriteDocBufRequest,
    ) -> Result<WriteDocBufResponse, docbuf_rpc::Status> {
        let doc_id = ctx
            .try_lock()
            .map_err(|_| docbuf_rpc::Status::ResourceUnavailable)?
            .write_docbuf(request)
            .map_err(|_| docbuf_rpc::Status::InvalidArguments)?;

        info!("Wrote DocBuf: {doc_id:?}");

        return Ok(WriteDocBufResponse { doc_id });
    }

    fn read_docbuf_rpc(
        ctx: Arc<Mutex<Self>>,
        request: ReadDocBufRequest,
    ) -> Result<ReadDocBufResponse, docbuf_rpc::Status> {
        let doc = ctx
            .try_lock()
            .map_err(|_| docbuf_rpc::Status::ResourceUnavailable)?
            .read_docbuf(request)
            .map_err(|_| docbuf_rpc::Status::InvalidArguments)?;

        info!("Read DocBuf: {doc:?}");

        return Ok(ReadDocBufResponse { doc });
    }

    fn update_docbuf_rpc(
        ctx: Arc<Mutex<Self>>,
        request: UpdateDocBufRequest,
    ) -> Result<UpdateDocBufResponse, docbuf_rpc::Status> {
        ctx.try_lock()
            .map_err(|_| docbuf_rpc::Status::ResourceUnavailable)?
            .update_docbuf(request)
            .map_err(|_| docbuf_rpc::Status::InvalidArguments)?;

        return Ok(UpdateDocBufResponse {});
    }

    fn delete_docbuf_rpc(
        ctx: Arc<Mutex<Self>>,
        request: DeleteDocBufRequest,
    ) -> Result<DeleteDocBufResponse, docbuf_rpc::Status> {
        let doc = ctx
            .try_lock()
            .map_err(|_| docbuf_rpc::Status::ResourceUnavailable)?
            .delete_docbuf(request)
            .map_err(|_| docbuf_rpc::Status::InvalidArguments)?;

        return Ok(DeleteDocBufResponse { doc });
    }

    fn count_docbuf_rpc(
        ctx: Arc<Mutex<Self>>,
        request: CountDocBufRequest,
    ) -> Result<CountDocBufResponse, docbuf_rpc::Status> {
        let count = ctx
            .try_lock()
            .map_err(|_| docbuf_rpc::Status::ResourceUnavailable)?
            .docbuf_count(request)
            .map_err(|_| docbuf_rpc::Status::InvalidArguments)?;

        return Ok(CountDocBufResponse { count });
    }

    // fn find_docbuf_rpc(
    //     ctx: Arc<Mutex<Self>>,
    //     request: FindDocBufRequest,
    // ) -> Result<FindDocBufResponse, docbuf_rpc::Status> {
    //     let doc = ctx
    //         .try_lock()
    //         .map_err(|_| docbuf_rpc::Status::ResourceUnavailable)?
    //         .find_docbuf(request)
    //         .map_err(|_| docbuf_rpc::Status::InvalidArguments)?;

    //     return Ok(FindDocBufResponse { doc });
    // }
}
