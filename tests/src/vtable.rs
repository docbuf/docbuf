use crate::complex::Document;

use docbuf_core::{
    traits::DocBuf,
    vtable::{VTable, VTableItem},
};

#[test]
fn test_vtable_serialization() -> Result<(), docbuf_db::Error> {
    let vtable = Document::vtable()?;

    let mut vtable_buf = Vec::new();
    vtable.write_to_buffer(&mut vtable_buf)?;

    let vtable2 = VTable::read_from_buffer(&mut vtable_buf)?;

    assert_eq!(*vtable, vtable2);

    Ok(())
}
