use crate::complex::Document;
use crate::rpc::{CERTIFICATE, PRIVATE_KEY, ROOT_CERTIFICATE, SERVER_PORT};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, net::SocketAddr};

use docbuf_core::deps::uuid::Uuid;
use docbuf_core::traits::DocBuf;
use docbuf_db::service::{CountDocBufRequest, WriteDocBufRequest};
use docbuf_db::DocBufDbManager;
use docbuf_db::PartitionKey;
use docbuf_db::Predicate;
use docbuf_db::Predicates;
use docbuf_db::{traits::*, DocBufDbRpcConfig};
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

#[test]
fn test_complex_db() -> Result<(), docbuf_db::Error> {
    let mut db = DocBufDbManager::from_config("/tmp/.docbuf/db/config.toml")?;

    // Set the tombstone flag to true.
    // This will zero out the data when a document is deleted,
    // instead of removing the document from the database,
    // and truncating the partition file.
    db.config = db.config.set_tombstone(true);

    let mut doc = Document::dummy();

    let id = doc.db_insert(&db)?;

    println!("Inserted Document: {:?}", id);
    assert_eq!(doc.uuid()?, id);

    let partition_key = doc.partition_key().ok();

    let doc_2 = Document::db_get(&db, id, partition_key.clone())?;

    println!("Found Document: {doc_2:?}");

    assert_eq!(doc_2.unwrap().uuid()?, id);

    let random_id = Uuid::new_v4().to_bytes_le();
    let random_doc = Document::db_get(&db, random_id, None::<PartitionKey>)?;
    assert_eq!(random_doc, None);

    println!("Random Document: {random_doc:?}");

    let mut doc_iter = Document::db_all(&db, None::<PartitionKey>)?;

    while let Some(doc_id) = doc_iter.next() {
        println!("Found Document ID: {doc_id:?}");
    }

    let doc_count = Document::db_count(&db, None::<PartitionKey>)?;

    println!("Document Count: {doc_count}");

    let vtable = Document::vtable()?;

    // println!("VTable {vtable:?}");

    let predicates = Predicates::from(Predicate::new(
        vtable
            .struct_by_name("Signature")?
            .field_by_name("signature")?,
        doc.metadata.signature.signature.as_slice(),
        std::cmp::Ordering::Equal,
    ))
    .and(Predicate::new(
        vtable
            .struct_by_name("Metadata")?
            .field_by_name("bool_data")?,
        (false as u8).to_le_bytes(),
        std::cmp::Ordering::Equal,
    ))
    .or(Predicate::new(
        vtable.struct_by_name("Document")?.field_by_name("author")?,
        ['A'; 16].map(|l| l as u8),
        std::cmp::Ordering::Equal,
    ));

    println!("Predicate: {predicates:?}");

    let mut doc_iter = Document::db_find(&db, predicates.clone(), None::<PartitionKey>)?;

    let mut count = 0;
    while let Some(doc) = doc_iter.next() {
        println!("Found Document ID: {:?}", doc.uuid()?);
        count += 1;
    }

    let count_where = Document::db_count_where(&db, predicates, None::<PartitionKey>)?;

    assert_eq!(count, count_where);

    // Update the author field of the document.
    doc.body = ['B'; 2048 / 2].iter().collect::<String>();

    // Update the document.
    doc.db_update(&db)?;

    println!("Updated Document Body");

    let previous_partition_key = doc.partition_key()?;

    // Update the document's partition key field.
    doc.author = ['A'; 100].iter().collect::<String>();

    println!("Updating Author");

    // Update the document.
    doc.db_update(&db)?;

    // Assert the partition key has changed, after the author has been updated.
    // This is because the partition key is derived from the author field,
    // as specified in the DocBuf annotations.
    assert_ne!(*previous_partition_key, *doc.partition_key()?);

    println!("Updated Document Author");

    let doc_3 =
        Document::db_get(&db, id, doc.partition_key().ok())?.expect("failed to get document");

    assert_eq!(doc_3.body, doc.body);

    let count = Document::db_count(&db, None::<PartitionKey>)?;

    println!("Count: {count}");

    // Delete the document.
    let doc_4 = doc.db_delete(&db)?;

    assert_eq!(doc_4.uuid()?, id);

    let count_2 = Document::db_count(&db, None::<PartitionKey>)?;

    assert_eq!(count_2, count - 1);

    assert!(Document::db_get(&db, id, doc_4.partition_key().ok())?.is_none());

    let vtable_ids = db.vtable_ids()?;

    for id in vtable_ids {
        println!("VTable: {id:?}");
    }

    Ok(())
}

#[tokio::test]
pub async fn test_db_rpc_server() -> Result<(), Box<dyn std::error::Error>> {
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

    let db = DocBufDbManager::from_config("/tmp/.docbuf/db/config.toml")?;
    let ctx = Arc::new(Mutex::new(db));
    let services = RpcServices::new(ctx).add_service(DocBufDbManager::rpc_service()?)?;

    RpcServer::bind(addr)?
        .start(services, Some(quic_config))
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn test_db_rpc_client() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .finish();

    // Add LEVEL filter to the subscriber to include DEBUG.
    println!("Log Level: {:?}", subscriber.max_level_hint());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    let mut db = DocBufDbManager::from_config("/tmp/.docbuf/db/config.toml")?;

    // Update the rpc configuration to connect to the server.
    db.config = db.config.set_rpc(DocBufDbRpcConfig {
        server: format!("[::1]:{SERVER_PORT}").parse().ok(),
        // cert_chain: Some(PathBuf::from(CERTIFICATE)),
        // priv_key: Some(PathBuf::from(PRIVATE_KEY)),
        // root_cert: Some(PathBuf::from(ROOT_CERTIFICATE)),
        ..Default::default()
    });

    // Attempt to establish a connection to the database.
    db.connect()?;

    let doc = Document::dummy();

    let id = doc.db_insert(&db)?;

    println!("Inserted Document: {:?}", id);

    // let doc_2 =
    //     Document::db_get(&db, id, doc.partition_key().ok())?.expect("failed to get document");

    // assert_eq!(doc_2.body, doc.body);

    let count = Document::db_count(&db, None::<PartitionKey>)?;

    println!("Count: {count}");

    Ok(())
}

#[tokio::test]
pub async fn test_write_docbuf_request() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::dummy();

    let vtable = Document::vtable()?;
    let mut buffer = vtable.alloc_buf();

    let offsets = doc.to_docbuf(&mut buffer)?;

    let request = WriteDocBufRequest {
        vtable_id: vtable.id().into(),
        partition_id: doc.partition_key()?.bucket(None),
        offsets: offsets.into(),
        buffer,
    };

    let req_vtable = WriteDocBufRequest::vtable()?;

    let mut req_buffer = req_vtable.alloc_buf();

    let req_offsets = request.to_docbuf(&mut req_buffer)?;

    // Deserialize the request.
    let mut req = WriteDocBufRequest::from_docbuf(&mut req_buffer)?;

    Ok(())
}

#[tokio::test]
pub async fn test_predicate_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::dummy();

    let vtable = Document::vtable()?;

    println!("VTable: {vtable:?}");

    let mut buffer = vtable.alloc_buf();

    let offsets = doc.to_docbuf(&mut buffer)?;

    let predicates = Predicates::from(Predicate::new(
        vtable
            .struct_by_name("Signature")?
            .field_by_name("signature")?,
        doc.metadata.signature.signature.as_slice(),
        std::cmp::Ordering::Equal,
    ));
    // .and(Predicate::new(
    //     vtable
    //         .struct_by_name("Metadata")?
    //         .field_by_name("bool_data")?,
    //     (false as u8).to_le_bytes(),
    //     std::cmp::Ordering::Equal,
    // ))
    // .or(Predicate::new(
    //     vtable.struct_by_name("Document")?.field_by_name("author")?,
    //     ['A'; 16].map(|l| l as u8),
    //     std::cmp::Ordering::Equal,
    // ));

    println!("Predicate: {predicates:?}\n\n");

    // let request = CountDocBufRequest {
    //     vtable_id: vtable.id().into(),
    //     partition_id: Some(doc.partition_key()?.bucket(None)),
    //     predicate: Some(predicates),
    // };

    let predicates_vtable = Predicates::vtable()?;

    println!("VTable: {predicates_vtable:?}\n\n");

    let mut predicates_buffer = predicates_vtable.alloc_buf();

    let predicates_offsets = predicates.to_docbuf(&mut predicates_buffer)?;

    println!("Predicates Buffer: {predicates_buffer:?}");

    println!("Preparing to deserialize predicates...\n\n");

    // Deserialize the request.
    let predicates_deserialized = Predicates::from_docbuf(&mut predicates_buffer)?;

    println!("Predicates: {predicates_deserialized:?}\n\n");

    assert_eq!(predicates.and.len(), predicates_deserialized.and.len());

    Ok(())
}
