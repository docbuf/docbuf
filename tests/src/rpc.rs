use super::{complex, strings};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, net::SocketAddr};

use ::serde::{Deserialize, Serialize};
use docbuf_core::serde;
use docbuf_core::traits::DocBuf;
use docbuf_rpc::RpcHeader;
use docbuf_rpc::{
    client::RpcClient,
    error::Error,
    quic::{QuicConfig, TlsOptions, MAX_QUIC_DATAGRAM_SIZE},
    server::RpcServer,
    service::{RpcMethodHandler, RpcService, RpcServices},
    RpcRequest, RpcResponse, RpcResult,
};
use tokio::{io::join, join};
use tracing::{level_filters::LevelFilter, Subscriber};

const SERVER_PORT: u16 = 60250;

// TODO: Replace quiche example certificates with self-signed certificates.
const CERTIFICATE: &str = "certs/cert.crt";
const PRIVATE_KEY: &str = "certs/cert.key";
const ROOT_CERTIFICATE: &str = "certs/rootca.crt";

fn complex_save_document<Ctx>(
    ctx: Ctx,
    req: RpcRequest<RpcHeader>,
) -> RpcResult<RpcResponse<RpcHeader>>
where
    Ctx: Clone + Send + Sync + 'static,
{
    unimplemented!("Service Method")
}

fn complex_get_document<Ctx>(
    ctx: Ctx,
    req: RpcRequest<RpcHeader>,
) -> RpcResult<RpcResponse<RpcHeader>>
where
    Ctx: Clone + Send + Sync + 'static,
{
    unimplemented!("Service Method")
}

#[tokio::test]
async fn test_rpc_server() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .finish();

    // Add LEVEL filter to the subscriber to include DEBUG.
    println!("Log Level: {:?}", subscriber.max_level_hint());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    let addr: SocketAddr = format!("[::1]:{SERVER_PORT}").parse()?;

    let mut quic_config = QuicConfig::development(TlsOptions::Server {
        cert_chain: PathBuf::from(CERTIFICATE),
        key: PathBuf::from(PRIVATE_KEY),
        boring_ctx_builder: None,
    })?;

    let _options = quic_config.as_mut();
    // options.verify_peer(false);

    println!("Spawning RPC Server");
    // IPv6 loopback address.
    // let complex_service = RpcServices::new();

    let ctx = Arc::new(Mutex::new(()));

    RpcServer::bind(addr, ctx, Some(quic_config))?
        .start(
            RpcServices::new().add_service(
                "complex",
                RpcService::<Arc<Mutex<()>>>::new()
                    .add_method("save_document", complex_save_document)?
                    .add_method("get_document", complex_get_document)?,
            )?,
        )
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

    let mut client = RpcClient::new(Option::<SocketAddr>::None, Some(quic_config))?;

    let server_name = Some("docbuf.com");
    client.connect(server_addr, server_name).await?;

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
        fn save_document(&self, doc: Document) -> RpcResult<Document> {
            unimplemented!()

            // Ok(Document {})
        }
    }

    Ok(())
}
