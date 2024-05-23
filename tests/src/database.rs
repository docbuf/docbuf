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
