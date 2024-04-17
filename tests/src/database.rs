use crate::complex::Document;

use docbuf_core::traits::DocBuf;
use docbuf_db::traits::*;
use docbuf_db::DocBufDbManager;

#[test]
fn test_complex_db() -> Result<(), docbuf_db::Error> {
    let db = DocBufDbManager::from_config("/tmp/.docbuf/db/config.toml")?;
    // let vtable = Document::vtable()?;

    // // Write the vtable to the database.
    // // This will create a file in the database directory, if it doesn't exist.
    // db.write_vtable(&vtable)?;

    // let vtable2 = db.read_vtable(&vtable.id())?;

    // assert_eq!(*vtable, vtable2);

    let doc = Document::dummy();

    let id = doc.db_insert(&db)?;
    let partition_key = doc.partition_key().ok(); // Some(doc.author.as_str());

    let doc = Document::db_get(&db, id, partition_key)?;

    println!("Doc: {:?}", doc);

    Ok(())
}
