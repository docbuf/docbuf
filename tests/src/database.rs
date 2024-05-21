use crate::complex::Document;

use docbuf_core::deps::uuid::Uuid;
use docbuf_core::traits::DocBuf;
use docbuf_db::traits::*;
use docbuf_db::DocBufDbManager;
use docbuf_db::PartitionKey;
use docbuf_db::Predicate;
use docbuf_db::Predicates;

#[test]
fn test_complex_db() -> Result<(), docbuf_db::Error> {
    let db = DocBufDbManager::from_config("/tmp/.docbuf/db/config.toml")?;

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

    let mut doc_iter = Document::db_find(&db, predicates, None::<PartitionKey>)?;

    while let Some(doc) = doc_iter.next() {
        println!("Found Document ID: {:?}", doc.uuid()?);
    }

    // Update the author field of the document.
    doc.body = ['B'; 2048 / 2].iter().collect::<String>();
    doc.db_update(&db)?;

    let doc_3 = Document::db_get(&db, id, partition_key)?.expect("failed to get document");

    assert_eq!(doc_3.body, doc.body);

    let count = Document::db_count(&db, None::<PartitionKey>)?;

    // Delete the document.
    let doc_4 = doc.db_delete(&db)?;

    assert_eq!(doc_4.uuid()?, id);

    let count_2 = Document::db_count(&db, None::<PartitionKey>)?;

    assert_eq!(count_2, count - 1);

    Ok(())
}
