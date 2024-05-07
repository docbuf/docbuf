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

#[docbuf_rpc]
impl ExampleService {
    fn save_document(
        _ctx: Arc<Mutex<()>>,
        mut document: complex::Document,
    ) -> Result<complex::Document, Error> {
        info!("Received a request to save a document: {document:?}");

        document.author = "Bob".to_string();

        Ok(document)
    }
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    #[docbuf {
        // Field validation
        // e.g., Regular expression validation.
        regex = "^[a-zA-Z ]{1,}$";
    }]
    pub name: String,
}

#[docbuf_rpc {
    // Optionally, disable the generation of the client interface.
    // Client generation is enabled by default.
    // gen_client = false;
}]
impl Hello {
    // User-Defined RPC Method Handler
    fn say_hello(_ctx: Arc<Mutex<()>>, mut doc: Hello) -> Result<Hello, docbuf_rpc::Error> {
        doc.name = "DocBuf RPC Goodbye Service".to_string();

        Ok(doc)
    }
}

// #[docbuf_rpc {
//     service = "hello";
// }]
// pub trait HelloClient {
//     // Define an RPC client method interface for the `Hello` service.
//     fn say_hello(
//         // provide a reference to the RPC client.
//         client: &RpcClient,
//         // provide the request DocBuf document.
//         doc: Hello,
//     ) -> Result<Hello, docbuf_rpc::Error>;
// }

// Compose multiple clients into a single client.
pub struct TestClient;

impl HelloClient for TestClient {}
impl ExampleServiceClient for TestClient {}

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
        .add_service(ExampleService::rpc_service()?)?
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

    let doc = complex::Document::dummy();

    let response = TestClient::save_document(&client, doc)?;

    println!("Document: {response:?}");

    let hello = Hello {
        name: "DocBuf RPC Client".to_string(),
    };

    for i in 0..10 {
        println!("Sending Request: {i}");
        let response = TestClient::say_hello(&client, hello.clone())?;
        println!("Hello: {response:?}");
    }

    // loop {}

    Ok(())
}
