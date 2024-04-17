pub mod partition_key;

pub use partition_key::*;

use crate::Error;

use std::{fs::File, path::PathBuf};

use docbuf_core::vtable::VTable;

pub struct Partition {
    file: File,
    vtable: VTable,
}

impl Partition {
    pub fn load(vtable: &VTable, path: impl Into<PathBuf>) -> Result<Self, Error> {
        let file = File::create(path.into())?;

        Ok(Self {
            file,
            vtable: vtable.to_owned(),
        })
    }

    pub fn write_docbuf(&self, buffer: Vec<u8>, offsets: Vec<u8>) -> Result<(), Error> {
        self.vtable.check_offsets(&offsets)?;

        unimplemented!("partition::Partition::write_docbuf")
    }
}
