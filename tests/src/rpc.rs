use super::{complex, strings};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, net::SocketAddr};

use ::serde::{Deserialize, Serialize};
use docbuf_core::serde;
use docbuf_core::traits::DocBuf;
use docbuf_rpc::{
    client::RpcClient,
    error::Error,
    quic::{QuicConfig, TlsOptions, MAX_QUIC_DATAGRAM_SIZE},
    server::RpcServer,
    service::{RpcMethodHandler, RpcService, RpcServices},
    RpcRequest, RpcResponse, RpcResult,
};
use docbuf_rpc::{RpcHeader, RpcHeaders};
use tokio::{io::join, join};
use tracing::info;
use tracing::{level_filters::LevelFilter, Subscriber};

const SERVER_PORT: u16 = 60250;

// TODO: Replace quiche example certificates with self-signed certificates.
const CERTIFICATE: &str = "certs/cert.crt";
const PRIVATE_KEY: &str = "certs/cert.key";
const ROOT_CERTIFICATE: &str = "certs/rootca.crt";

pub struct ExampleService;

impl ExampleService {
    fn complex_save_document<Ctx>(_ctx: Ctx, mut req: RpcRequest) -> RpcResult
    where
        Ctx: Send + Sync + 'static,
    {
        let mut document = req.from_docbuf::<complex::Document>()?;

        info!("Received a request {req:?} to save a document: {document:?}");

        document.author = "Alice".to_string();

        let mut response = RpcResponse::with_empty_body(req.stream_id, req.headers);
        document.to_docbuf(&mut response.body)?;

        Ok(response)
    }

    fn complex_get_document<Ctx>(ctx: Ctx, mut req: RpcRequest) -> RpcResult
    where
        Ctx: Send + Sync + 'static,
    {
        let document = req.from_docbuf::<complex::Document>()?;

        unimplemented!("Service Method")
    }

    fn service<Ctx>() -> Result<RpcService<Ctx>, Error>
    where
        Ctx: Send + Sync + 'static,
    {
        RpcService::<Ctx>::new("complex")
            .add_method("save_document", Self::complex_save_document)?
            .add_method("get_document", Self::complex_get_document)
    }
}

#[tokio::test]
async fn test_rpc_server() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .finish();

    // Add LEVEL filter to the subscriber to include DEBUG.
    // println!("Log Level: {:?}", subscriber.max_level_hint());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    let addr: SocketAddr = format!("[::1]:{SERVER_PORT}").parse()?;

    let mut quic_config = QuicConfig::development(TlsOptions::Server {
        cert_chain: PathBuf::from(CERTIFICATE),
        key: PathBuf::from(PRIVATE_KEY),
        boring_ctx_builder: None,
    })?;

    let ctx = Arc::new(Mutex::new(()));

    let services = RpcServices::new(ctx).add_service(ExampleService::service()?)?;

    RpcServer::bind(addr)?
        .start(services, Some(quic_config))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_rpc_client() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .finish();

    // Add LEVEL filter to the subscriber to include DEBUG.
    println!("Log Level: {:?}", subscriber.max_level_hint());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    let server_addr: SocketAddr = format!("[::1]:{SERVER_PORT}").parse()?;

    println!("Spawning RPC Client");
    println!("Connecting to Server: {}", server_addr);

    let mut quic_config = QuicConfig::development(TlsOptions::Client {
        cert_chain: PathBuf::from(CERTIFICATE),
        key: PathBuf::from(PRIVATE_KEY),
    })?;
    let options = quic_config.as_mut();
    options.load_verify_locations_from_file(ROOT_CERTIFICATE)?;
    // options.verify_peer(false);

    // let mut client = RpcClient::bind(Option::<SocketAddr>::None)?;

    let server_name = Some("docbuf.com");

    let client = RpcClient::connect(server_addr, server_name, Some(quic_config))?;

    let doc = complex::Document::dummy();
    let mut buffer = complex::Document::vtable()?.alloc_buf();
    doc.to_docbuf(&mut buffer)?;

    info!("Sending Test Request");

    // Sleep for 5 seconds.
    // std::thread::sleep(std::time::Duration::from_secs(5));

    // Send a test request.
    let request = RpcRequest::default()
        .add_headers(
            vec![
                RpcHeader::new(b":method", b"POST"),
                RpcHeader::new(b":scheme", b"http"),
                RpcHeader::new(b":authority", b"localhost"),
                RpcHeader::new(b":path", b"/complex/save_document"),
                RpcHeader::new(b"content-type", b"application/docbuf+rpc"),
                RpcHeader::new(b"user-agent", b"docbuf-rpc-client/0.1.0"),
                RpcHeader::new(b"content-length", &buffer.len().to_string().as_bytes()),
            ]
            .into(),
        )
        .add_body(buffer.clone());

    let response = client.send(request).await?;

    info!("Response: {response:?}");

    Ok(())
}

#[test]
fn test_rpc_macro() -> Result<(), docbuf_core::error::Error> {
    use docbuf_core::traits::DocBuf;
    use docbuf_macros::*;
    use docbuf_rpc::RpcResult;

    #[docbuf]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Document {}

    // Create a service for the Document struct.
    pub struct DocumentService {}

    // #[docbuf_rpc {
    //     service = "complex";
    // }]
    impl DocumentService {
        fn save_document(&self, doc: Document) -> RpcResult {
            unimplemented!()

            // Ok(Document {})
        }
    }

    Ok(())
}
