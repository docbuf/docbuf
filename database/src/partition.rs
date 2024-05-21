pub mod partition_id;
pub mod partition_key;

pub use partition_id::*;
pub use partition_key::*;

use crate::{Error, Predicates};

use std::{
    cmp::Ordering,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
    path::PathBuf,
    sync::mpsc::channel,
};

use docbuf_core::vtable::{VTable, VTableFieldOffsets};
use file_lock::{FileLock, FileOptions};

/// The offset of the partition count in the partition file.
const PARTITION_COUNT_OFFSET: u64 = 8;

pub enum PartitionPermission {
    Write,
    Update,
    Read,
}

impl Into<FileOptions> for PartitionPermission {
    fn into(self) -> FileOptions {
        let options = FileOptions::new();

        match self {
            Self::Write => options.write(true).read(true).create(true).append(true),
            Self::Read => options.read(true),
            Self::Update => options.read(true).write(true),
        }
    }
}

pub struct Partition {
    id: PartitionId,
    lock: FileLock,
    vtable: VTable,
}

impl Partition {
    /// Locks a partition file and returns a Partition object.
    pub fn load(
        vtable: &VTable,
        path: impl Into<PathBuf>,
        partition_id: PartitionId,
        permission: PartitionPermission,
    ) -> Result<Self, Error> {
        let block_if_locked = true;
        let lock = FileLock::lock(path.into(), block_if_locked, permission.into())?;

        Ok(Self {
            id: partition_id,
            lock,
            vtable: vtable.to_owned(),
        })
    }

    /// Returns the partition u16 identifier.
    pub fn id(&self) -> PartitionId {
        self.id.to_owned()
    }

    // Reads the first 8 bytes of the partition file to get the count of
    // the number of docbufs in the partition, then increments the count
    // by 1 and writes the new count back to the partition file.
    fn increment_count(&mut self) -> Result<(), Error> {
        let mut count = [0u8; 8];
        self.lock.file.seek(SeekFrom::Start(0))?;
        self.lock.file.read_exact(&mut count)?;

        let mut count = u64::from_le_bytes(count);
        count += 1;

        self.lock.file.write_at(&count.to_le_bytes(), 0)?;

        Ok(())
    }

    /// Decrement the count of the number of docbufs in the partition.
    fn decrement_count(&mut self) -> Result<(), Error> {
        let mut count = [0u8; 8];
        self.lock.file.seek(SeekFrom::Start(0))?;
        self.lock.file.read_exact(&mut count)?;

        let mut count = u64::from_le_bytes(count);
        count -= 1;

        self.lock.file.write_at(&count.to_le_bytes(), 0)?;

        Ok(())
    }

    /// Writes a document buffer to the partition file. This will append
    /// the docbuf to the end of the file, prefixed by the docbuf's
    /// field offsets.
    ///
    /// | offset length | offsets | docbuf | ... next docbuf offset length ...
    ///
    pub fn write_docbuf(&mut self, mut offsets: Vec<u8>, buffer: Vec<u8>) -> Result<(), Error> {
        println!("Writing docbuf length {} to partition file.", buffer.len());

        self.vtable.check_offsets(&offsets)?;

        // Extend the offset bytes by the docbuf buffer.
        offsets.extend(buffer.as_slice());

        // If the partition file is empty, prepend the file with the count
        // of the number of docbufs in the partition.
        let is_empty = self.lock.file.metadata()?.len() == 0;

        if is_empty {
            let count = 1u64.to_le_bytes();
            self.lock.file.write_all(&count)?;
        }

        // Write the offset length of offset bytes.
        self.lock.file.write_all(&offsets)?;

        if !is_empty {
            // increment the count of the partition file.
            self.increment_count()?;
        }

        Ok(())
    }

    /// Reads a document buffer from the partition file. This will read
    /// the offsets and the docbuf.
    /// Will return the size of the document buffer.
    pub fn read_docbuf(&mut self, doc_id: &[u8; 16]) -> Result<Option<Vec<u8>>, Error> {
        println!("Reading docbuf from partition file.");
        // Offset length is fixed size according to the vtable.
        let offset_length = self.vtable.offset_len();
        let file_length = self.lock.file.metadata()?.len();

        let mut buffer = self.vtable.alloc_buf();

        self.lock
            .file
            .seek(SeekFrom::Start(PARTITION_COUNT_OFFSET))?;

        loop {
            let cursor_pos = self.lock.file.stream_position()?;

            // Check if the cursor is at the end of the file.
            if cursor_pos >= file_length {
                break;
            }

            println!("Reading offset");

            // Read the document buffer offsets from the partition.
            self.lock.file.read_exact(&mut buffer[..offset_length])?;

            let offsets = VTableFieldOffsets::from_bytes(&buffer[..offset_length]);
            let doc_buffer_len = offsets.doc_buffer_len();

            println!("Reading docbuf length {}", doc_buffer_len);

            self.lock.file.read_exact(&mut buffer[..doc_buffer_len])?;

            // Check if the first 16 bytes of the buffer match the doc_id.
            // If so, return the document buffer.
            if buffer[..16] == *doc_id {
                return Ok(Some(buffer[..doc_buffer_len].to_vec()));
            }
        }

        // Did not find a document buffer in the partition file.
        Ok(None)
    }

    /// Delete the document buffer from the partition file.
    pub fn delete_docbuf(&mut self, doc_id: [u8; 16]) -> Result<Vec<u8>, Error> {
        println!("Deleting docbuf from partition file.");
        let file_length = self.lock.file.metadata()?.len();
        let offset_length = self.vtable.offset_len();

        let mut buffer = self.vtable.alloc_buf();

        self.lock
            .file
            .seek(SeekFrom::Start(PARTITION_COUNT_OFFSET))?;

        loop {
            let cursor_pos = self.lock.file.stream_position()?;

            // Check if the cursor is at the end of the file.
            if cursor_pos >= file_length {
                break;
            }

            println!("Reading offset");

            // Read the document buffer offsets from the partition.
            self.lock.file.read_exact(&mut buffer[..offset_length])?;
            let offsets = VTableFieldOffsets::from_bytes(&buffer[..offset_length]);

            let doc_buffer_len = offsets.doc_buffer_len();

            println!("Reading docbuf length {}", doc_buffer_len);

            self.lock.file.read_exact(&mut buffer[..doc_buffer_len])?;

            // Check if the first 16 bytes of the buffer match the doc_id.
            // If so, return the document buffer.
            if buffer[..16] == doc_id {
                println!("Found Document, deleting...");
                let section_end = self.lock.file.stream_position()?;

                println!("Section End: {section_end:?}");

                let section_start = section_end - (offset_length + doc_buffer_len) as u64;

                println!("Section Start: {section_start:?}");

                let shift_length = doc_buffer_len + offset_length;
                let mut shift_buffer = vec![0u8; 1024];

                self.lock.file.seek(SeekFrom::Start(section_end))?;

                let mut remaining_bytes = file_length - section_end;

                while remaining_bytes > 0 {
                    let cursor = self.lock.file.stream_position()?;

                    let read_length = std::cmp::min(remaining_bytes, shift_buffer.len() as u64);

                    self.lock
                        .file
                        .read_exact(&mut shift_buffer[..read_length as usize])?;

                    self.lock.file.seek(SeekFrom::Start(cursor - read_length))?;

                    self.lock
                        .file
                        .write_all(&shift_buffer[..read_length as usize])?;

                    remaining_bytes -= read_length;

                    self.lock.file.seek(SeekFrom::Start(cursor + read_length))?;
                }

                // Truncate the file to the new length.
                self.lock.file.set_len(file_length - shift_length as u64)?;

                // Check the file length matches the updated length.
                let new_file_length = self.lock.file.metadata()?.len();
                let expected_length = file_length - shift_length as u64;

                println!("New File Length: {new_file_length}");
                println!("Expected Length: {expected_length}");

                if new_file_length != expected_length {
                    assert_eq!(new_file_length, expected_length, "File length mismatch");
                }

                // Decrement the count of the partition file.
                self.decrement_count()?;

                return Ok(buffer[..doc_buffer_len].to_vec());
            }
        }

        // Did not find a document buffer in the partition file.
        buffer.clear();
        Ok(buffer)
    }

    /// Updates a document buffer in the partition file. This will read
    /// the offsets and the docbuf, then write the new docbuf to the
    /// partition file, shifting the remaining file bytes either to the left
    /// if the new docbuf is shorter in length, or shifting to the right if
    /// the docbuf is larger than the original docbuf. The offset length will
    /// remain the same, but the contents of the offset are updated in the partition file,
    /// corresponding to the new docbuf.
    pub fn update_docbuf(
        &mut self,
        doc_id: [u8; 16],
        offsets: Vec<u8>,
        docbuf: Vec<u8>,
    ) -> Result<(), Error> {
        println!("update_docbuf::Updating Document Buffer");
        let file_length = self.lock.file.metadata()?.len();

        println!("File Length: {file_length:?}");

        let offset_length = self.vtable.offset_len();

        let mut buffer = self.vtable.alloc_buf();

        self.lock
            .file
            .seek(SeekFrom::Start(PARTITION_COUNT_OFFSET))?;

        loop {
            let cursor_pos = self.lock.file.stream_position()?;

            println!("Cursor Position: {cursor_pos:?}");

            // Check if the cursor is at the end of the file.
            if cursor_pos >= file_length {
                break;
            }

            // Read the document buffer offsets from the partition.
            self.lock.file.read_exact(&mut buffer[..offset_length])?;

            let doc_buffer_len =
                VTableFieldOffsets::from_bytes(&buffer[..offset_length]).doc_buffer_len();

            self.lock.file.read_exact(&mut buffer[..doc_buffer_len])?;

            if buffer[..16] == doc_id {
                println!("Found Document, updating...");
                let section_end = self.lock.file.stream_position()?;

                println!("Section End: {section_end:?}");

                let section_start = section_end - (offset_length + doc_buffer_len) as u64;

                println!("Section Start: {section_start:?}");

                match docbuf.len().cmp(&doc_buffer_len) {
                    Ordering::Equal => {
                        // Write the new offsets and docbuf to the partition file.
                        self.lock.file.seek(SeekFrom::Start(section_start))?;
                        self.lock.file.write_all(&offsets)?;
                        self.lock.file.write_all(&docbuf)?;
                    }
                    Ordering::Less => {
                        println!("Less Than");
                        let shift_length = doc_buffer_len - docbuf.len();
                        let mut shift_buffer = vec![0u8; 1024];

                        self.lock.file.seek(SeekFrom::Start(section_end))?;

                        let mut remaining_bytes = file_length - section_end;

                        while remaining_bytes > 0 {
                            let cursor = self.lock.file.stream_position()?;

                            let read_length =
                                std::cmp::min(remaining_bytes, shift_buffer.len() as u64);

                            self.lock
                                .file
                                .read_exact(&mut shift_buffer[..read_length as usize])?;

                            self.lock.file.seek(SeekFrom::Start(cursor - read_length))?;

                            self.lock
                                .file
                                .write_all(&shift_buffer[..read_length as usize])?;

                            remaining_bytes -= read_length;

                            self.lock.file.seek(SeekFrom::Start(cursor + read_length))?;
                        }

                        // Write the new offsets and docbuf to the partition file.
                        self.lock.file.seek(SeekFrom::Start(section_start))?;
                        self.lock.file.write_all(&offsets)?;
                        self.lock.file.write_all(&docbuf)?;

                        // Truncate the file to the new length.
                        self.lock.file.set_len(file_length - shift_length as u64)?;

                        // Check the file length matches the updated length.
                        let new_file_length = self.lock.file.metadata()?.len();
                        let expected_length = file_length - shift_length as u64;

                        println!("New File Length: {new_file_length}");
                        println!("Expected Length: {expected_length}");

                        if new_file_length != expected_length {
                            assert_eq!(new_file_length, expected_length, "File length mismatch");
                        }
                    }
                    Ordering::Greater => {
                        println!("Greater Than");
                        // Shift the remaing bytes to the right, by the
                        // difference in length between the new docbuf and the
                        // original docbuf.
                        let shift_length = docbuf.len() - doc_buffer_len;
                        let mut shift_buffer = vec![0u8; 1024];

                        // Add the shift length to the end of the file.
                        self.lock.file.set_len(file_length + shift_length as u64)?;

                        // Set the cursor to the end of the file
                        self.lock.file.seek(SeekFrom::End(0))?;

                        let mut remaining_bytes = file_length - section_end;
                        // let mut cursor = file_length;

                        while remaining_bytes > 0 {
                            println!("Remaining Bytes: {remaining_bytes}");
                            // Shift the bytes to the right from the end
                            // of the file, until the section end plus
                            // the shift length is reached.
                            let read_length =
                                std::cmp::min(remaining_bytes, shift_buffer.len() as u64);

                            self.lock
                                .file
                                .seek(SeekFrom::Current(-(read_length as i64)))?;

                            let cursor = self.lock.file.stream_position()?;

                            self.lock
                                .file
                                .read_exact(&mut shift_buffer[..read_length as usize])?;

                            self.lock
                                .file
                                .seek(SeekFrom::Start(cursor + shift_length as u64))?;

                            self.lock
                                .file
                                .write_all(&shift_buffer[..read_length as usize])?;

                            remaining_bytes -= read_length;

                            // Shift back to the cursor position.
                            self.lock.file.seek(SeekFrom::Start(cursor))?;
                        }

                        // Write the new offsets and docbuf to the partition file.
                        self.lock.file.seek(SeekFrom::Start(section_start))?;

                        println!(
                            "Writing offsets length {} at position: {}",
                            offsets.len(),
                            self.lock.file.stream_position()?,
                        );

                        self.lock.file.write_all(&offsets)?;

                        // Write the new offsets and docbuf to the partition file.
                        self.lock
                            .file
                            .seek(SeekFrom::Start(section_start + offsets.len() as u64))?;

                        println!(
                            "Writing docbuf length {} at position: {}",
                            docbuf.len(),
                            self.lock.file.stream_position()?,
                        );

                        self.lock.file.write_all(&docbuf)?;

                        self.lock.file.flush()?;

                        // Check the file length matches the updated length.
                        let new_file_length = self.lock.file.metadata()?.len();
                        let expected_length = file_length + shift_length as u64;

                        println!("New File Length: {new_file_length}");
                        println!("Expected Length: {expected_length}");

                        if new_file_length != expected_length {
                            assert_eq!(new_file_length, expected_length, "File length mismatch");
                        }
                    }
                }

                // Break after processing the update for the target docbuf.
                break;
            }
        }

        Ok(())
    }

    /// Reads the first 8 bytes of the partition file to get the count of
    /// the number of docbufs in the partition.
    pub fn count(&mut self) -> Result<usize, Error> {
        let file_length = self.lock.file.metadata()?.len();

        match file_length == 0 {
            // Return zero if the partition file is empty.
            true => Ok(0),
            // Otherwise, read the count from the partition file.
            false => {
                let mut count = [0u8; 8];
                self.lock.file.seek(SeekFrom::Start(0))?;
                self.lock.file.read_exact(&mut count)?;

                Ok(u64::from_le_bytes(count) as usize)
            }
        }
    }

    pub fn search_docbufs(
        &mut self,
        predicates: &Predicates,
    ) -> Result<impl Iterator<Item = Vec<u8>>, Error> {
        let offset_length = self.vtable.offset_len();
        let file_length = self.lock.file.metadata()?.len();

        let mut buffer = self.vtable.alloc_buf();
        let (tx, rx) = channel();

        self.lock
            .file
            .seek(SeekFrom::Start(PARTITION_COUNT_OFFSET))?;

        loop {
            let cursor_pos = self.lock.file.stream_position()?;

            // Check if the cursor is at the end of the file.
            if cursor_pos >= file_length {
                break;
            }

            // Read the document buffer offsets from the partition.
            self.lock.file.read_exact(&mut buffer[..offset_length])?;

            let offsets = VTableFieldOffsets::from_bytes(&buffer[..offset_length]);
            let doc_buffer_len = offsets.doc_buffer_len();

            self.lock.file.read_exact(&mut buffer[..doc_buffer_len])?;

            // Evaluate the predicates on the document buffer.
            if !predicates.eval(&self.vtable, &offsets, &buffer[..doc_buffer_len])? {
                // Return early if the predicates do not match.
                continue;
            }

            tx.send(buffer[..doc_buffer_len].to_vec()).unwrap();
        }

        Ok(rx.into_iter())
    }

    /// Read the DocBuf IDs from the partition file.
    pub fn read_docbuf_ids(&mut self) -> Result<impl Iterator<Item = [u8; 16]>, Error> {
        let offset_length = self.vtable.offset_len();
        let file_length = self.lock.file.metadata()?.len();

        let mut buffer = self.vtable.alloc_buf();
        let (tx, rx) = channel();

        self.lock
            .file
            .seek(SeekFrom::Start(PARTITION_COUNT_OFFSET))?;

        loop {
            let cursor_pos = self.lock.file.stream_position()?;

            // Check if the cursor is at the end of the file.
            if cursor_pos >= file_length {
                break;
            }

            // Read the document buffer offsets from the partition.
            self.lock.file.read_exact(&mut buffer[..offset_length])?;

            let offsets = VTableFieldOffsets::from_bytes(&buffer[..offset_length]);
            let doc_buffer_len = offsets.doc_buffer_len();

            self.lock.file.read_exact(&mut buffer[..doc_buffer_len])?;

            // Extract the doc_id from the buffer.
            tx.send([
                buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13],
                buffer[14], buffer[15],
            ])
            .map_err(|e| eprintln!("Failed to send ID"))
            .ok();
        }

        Ok(rx.into_iter())
    }
}
