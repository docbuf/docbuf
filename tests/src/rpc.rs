use super::{complex, strings};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, net::SocketAddr};

use ::serde::{Deserialize, Serialize};
use docbuf_core::serde;
use docbuf_core::traits::DocBuf;
use docbuf_macros::*;
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
    fn save_document<Ctx>(
        _ctx: Ctx,
        mut document: complex::Document,
    ) -> Result<complex::Document, Error>
    where
        Ctx: Send + Sync + 'static,
    {
        info!("Received a request to save a document: {document:?}");

        document.author = "Bob".to_string();

        Ok(document)
    }

    fn complex_save_document<Ctx>(_ctx: Ctx, mut req: RpcRequest) -> RpcResult
    where
        Ctx: Send + Sync + 'static,
    {
        let document = Self::save_document(_ctx, req.as_docbuf::<complex::Document>()?)?;

        // Instantiate the response content body buffer to the length of the content length.
        let mut buffer = Vec::new();

        document.to_docbuf(&mut buffer)?;

        let headers = RpcHeaders::default()
            .with_path(req.headers.path()?)
            .with_content_length(buffer.len());

        Ok(RpcResponse::new(req.stream_id, headers, buffer))
    }

    fn complex_get_document<Ctx>(ctx: Ctx, mut req: RpcRequest) -> RpcResult
    where
        Ctx: Send + Sync + 'static,
    {
        let document = req.as_docbuf::<complex::Document>()?;

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

#[docbuf]
#[derive(Debug, Serialize, Deserialize)]
pub struct Hello {
    #[docbuf {
        // Field validation
        // e.g., Regular expression validation.
        regex = "^[a-zA-Z ]{1,}$";
    }]
    pub name: String,
}

#[docbuf_rpc]
impl Hello {
    // User-Defined RPC Method Handler
    fn say_hello(ctx: Arc<Mutex<()>>, mut doc: Hello) -> Result<Hello, docbuf_rpc::Error> {
        doc.name = "DocBuf RPC Hello Service".to_string();

        Ok(doc)
    }
}

#[docbuf_rpc {
    service = "hello";
}]
pub trait HelloClient {
    // Define an RPC client method interface for the `Hello` service.
    fn say_hello(
        // provide a reference to the RPC client.
        client: &RpcClient,
        // provide the request DocBuf document.
        doc: Hello,
    ) -> Result<Hello, docbuf_rpc::Error>;
}

pub struct TestClient;

impl HelloClient for TestClient {}

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

    let quic_config = QuicConfig::development(TlsOptions::Server {
        cert_chain: PathBuf::from(CERTIFICATE),
        key: PathBuf::from(PRIVATE_KEY),
        boring_ctx_builder: None,
    })?;

    let ctx = Arc::new(Mutex::new(()));

    let services = RpcServices::new(ctx)
        .add_service(ExampleService::service()?)?
        // Add the Hello RPC Service generated from the macro.
        .add_service(Hello::rpc_service()?)?;

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

    let hello = Hello {
        name: "DocBuf RPC Client".to_string(),
    };

    let response = TestClient::say_hello(&client, hello)?;

    println!("Hello: {response:?}");

    // let doc = complex::Document::dummy();
    // let mut buffer = complex::Document::vtable()?.alloc_buf();
    // doc.to_docbuf(&mut buffer)?;

    // info!("Sending Test Request: {buffer:?}");

    // // Sleep for 5 seconds.
    // // std::thread::sleep(std::time::Duration::from_secs(5));

    // // Send a test request.
    // let request = RpcRequest::default()
    //     .add_headers(
    //         vec![
    //             RpcHeader::new(b":method", b"POST"),
    //             RpcHeader::new(b":scheme", b"http"),
    //             RpcHeader::new(b":authority", b"localhost"),
    //             RpcHeader::new(b":path", b"/complex/save_document"),
    //             RpcHeader::new(b"content-type", b"application/docbuf+rpc"),
    //             RpcHeader::new(b"user-agent", b"docbuf-rpc-client/0.1.0"),
    //             RpcHeader::new(b"content-length", &buffer.len().to_string().as_bytes()),
    //         ]
    //         .into(),
    //     )
    //     .add_body(buffer.clone());

    // let mut response = client.send(request)?;

    // info!("Response: {response:?}");

    // let doc = complex::Document::from_docbuf(&mut response.body).expect("Failed to parse document");

    // info!("Document: {doc:?}");

    Ok(())
}
