use docbuf_rpc::{
    client::RpcClient,
    error::Error,
    quic::{QuicConfig, TlsOptions, MAX_QUIC_DATAGRAM_SIZE},
    server::RpcServer,
};
use tokio::{io::join, join};
use tracing::{level_filters::LevelFilter, Subscriber};

use std::net::SocketAddr;
use std::path::PathBuf;

const SERVER_PORT: u16 = 60250;

// TODO: Replace quiche example certificates with self-signed certificates.
const CERTIFICATE: &str = "certs/cert.crt";
const PRIVATE_KEY: &str = "certs/cert.key";
const ROOT_CERTIFICATE: &str = "certs/rootca.crt";

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
    let mut server = RpcServer::bind(addr, Some(quic_config))?;

    println!("Server Socket Address: {}", server.address()?);
    server.start().await?;

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
