use crate::{SetTestValues, TestHarness};

use std::collections::HashMap;

use docbuf_core::{
    traits::{DocBuf, DocBufMap},
    uuid::Uuid,
    vtable::{VTable, VTableId, VTABLE_FIELD_OFFSET_SIZE_BYTES},
};
use docbuf_db::{traits::*, PartitionKey};
use docbuf_macros::*;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
pub struct Complex(Vec<Document>);

#[docbuf {
    namespace: "complex";
    // Sign the entire document, will create an allocation for the document
    // signature
    sign = true;
    // Use the ed25519 signature algorithm
    // crypto = "ed25519";
    // Use the sha256 hash algorithm
    // hash = "sha256";
    html = "path/to/html/template.html";
    // Path to the database configuration file.
    // This will automatically add a `_uuid` field to the document
    db_config = "/tmp/.docbuf/db/config.toml";
    // To add a `_uuid` field to the document separately, set:
    // uuid = true;
}]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Document {
    #[docbuf {
        // Hash the author string and use it as the partition key,
        // rather than the uuid. This will put the author in the same
        // partition.
        partition_key = true;
    }]
    pub author: String,
    pub title: String,
    #[docbuf {
        min_length = 0;
        max_length = 4096;
    }]
    pub body: String,
    pub footer: String,
    // Automatically create signature allocation for the footer
    #[docbuf {
        sign = true;
        crypto = "ed25519";
        hash = "sha256";
    }]
    pub metadata: Metadata,
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body && self.footer == other.footer && self.metadata == other.metadata
    }
}

#[docbuf {
    sign = true;
}]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    #[docbuf {
        min_length = 0;
    }]
    pub description: String,
    pub signature: Signature,
    pub u8_data: u8,
    pub u16_data: u16,
    pub u32_data: u32,
    pub u64_data: u64,
    pub u128_data: u128,
    pub usize_data: usize,
    pub f32_data: f32,
    pub f64_data: f64,
    pub i8_data: i8,
    pub i16_data: i16,
    pub i32_data: i32,
    pub i64_data: i64,
    pub i128_data: i128,
    pub isize_data: isize,
    pub hash_map_data: std::collections::HashMap<String, String>,
    #[serde(with = "serde_bytes")]
    pub byte_data: Vec<u8>,
    pub bool_data: bool,
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
            && self.u8_data == other.u8_data
            && self.u16_data == other.u16_data
            && self.u32_data == other.u32_data
            && self.u64_data == other.u64_data
            && self.u128_data == other.u128_data
            && self.usize_data == other.usize_data
            && self.f32_data == other.f32_data
            && self.f64_data == other.f64_data
            && self.i8_data == other.i8_data
            && self.i16_data == other.i16_data
            && self.i32_data == other.i32_data
            && self.i64_data == other.i64_data
            && self.i128_data == other.i128_data
            && self.isize_data == other.isize_data
            && self.hash_map_data == other.hash_map_data
            && self.signature == other.signature
            && self.byte_data == other.byte_data
            && self.bool_data == other.bool_data
    }
}

// #[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf {
    // sign = "true";
}]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Signature {
    #[docbuf {
        length = 32;
    }]
    #[serde(with = "serde_bytes")]
    pub signature: [u8; 32],
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.signature == other.signature
    }
}

impl Document {
    pub fn dummy() -> Self {
        Self {
            _uuid: Uuid::new_v4(),
            author: ["A"; 16].concat(),
            title: ["T"; 64].concat(),
            body: ["B"; 2048].concat(),
            footer: ["F"; 32].concat(),
            metadata: Metadata {
                description: ["D"; 512].concat(),
                signature: Signature { signature: [0; 32] },
                u8_data: u8::MAX,
                u16_data: u16::MAX,
                u32_data: u32::MAX,
                u64_data: u64::MAX,
                u128_data: u128::MAX,
                usize_data: usize::MAX,
                f32_data: f32::MAX,
                f64_data: f64::MAX,
                i8_data: i8::MIN,
                i16_data: i16::MIN,
                i32_data: i32::MIN,
                i64_data: i64::MIN,
                i128_data: i128::MIN,
                isize_data: isize::MIN,
                hash_map_data: (|| {
                    let mut map = std::collections::HashMap::new();
                    map.insert("0".to_string(), "0".to_string());
                    map.insert("1".to_string(), "1".to_string());
                    map.insert("2".to_string(), "2".to_string());
                    map.insert("3".to_string(), ["3"; 500].concat());
                    map
                })(),
                byte_data: (|| {
                    let mut data = Vec::with_capacity(255);
                    for i in 0..255 {
                        data.push(i as u8);
                    }
                    data
                })(),
                bool_data: true,
            },
        }
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SetTestValues for Document {}

impl<'de> TestHarness<'de> for Document {}

#[test]
fn test_serialize_complex() -> Result<(), docbuf_core::error::Error> {
    let document = Document::dummy();

    let mut buffer = Vec::with_capacity(1024);

    println!("document: {:?}", document);

    document
        // Round Trip Test
        .assert_serialization_round_trip(&mut buffer)
        .expect("Failed round trip serialization")
        // Serialization Size Comparison Test
        .assert_serialization_size(&mut buffer)
        .expect("Failed encoding benchmark");

    let doc = Document::from_docbuf(&mut buffer)?;

    println!("doc: {:?}", doc);

    // assert_eq!(document, doc);

    Ok(())
}

#[test]
fn test_serialize_hash_map() -> Result<(), docbuf_core::error::Error> {
    let mut map = std::collections::HashMap::new();
    map.insert("0".to_string(), "0".to_string());
    map.insert("1".to_string(), "1".to_string());
    map.insert("2".to_string(), "2".to_string());
    map.insert("3".to_string(), ["3"; 1024].concat());

    let mut buffer = Vec::with_capacity(1024);

    bincode::serialize_into(&mut buffer, &map).expect("Failed to serialize");

    println!("Bincode Buffer: {:?}", buffer);
    println!("Buffer length: {:?}", buffer.len());

    //

    Ok(())
}

#[test]
fn test_write_file() -> Result<(), docbuf_core::error::Error> {
    let doc = Document::dummy();

    let path_to_file = "test.dbuf"; // relative to current working directory

    doc.to_file(path_to_file)?;

    Ok(())
}

#[test]
fn test_docbuf_map() -> Result<(), docbuf_core::error::Error> {
    let document = Document::dummy();

    let mut buffer = Vec::with_capacity(1024);

    println!("document: {:?}\n\n", document);

    let mut offsets = document.to_docbuf(&mut buffer)?;

    println!("Buffer: {:?}\n\n", buffer);

    println!("Offsets: {:?}\n\n", offsets);

    let target_offset = offsets
        .as_ref()
        .get(1)
        .map(|offset| offset.clone())
        .unwrap();

    let mut field_data: String = Document::vtable()?.docbuf_map(&buffer, &target_offset)?;

    println!("Field: {:?}\n\n", field_data);

    assert_eq!(document.author, field_data);

    field_data = String::from("Hello, World!");
    let target_offset = Document::vtable()?.docbuf_map_replace(
        &field_data,
        // consumes the target offset, and returns the updated offset
        target_offset,
        // mutates the buffer in place
        &mut buffer,
        // mutates the offsets in place
        &mut offsets,
    )?;

    let doc = Document::from_docbuf(&mut buffer)?;

    assert_eq!(field_data, doc.author);

    Ok(())
}

#[test]
fn test_vtable_size() -> Result<(), docbuf_core::error::Error> {
    let vtable = Document::vtable()?;

    // Get the derministic 8-byte id of the vtable
    let vid = vtable.id().to_string();

    println!("VTable ID: {:?}", vid);

    let page_size = vtable.page_size(None);
    let avg_size = vtable.avg_size() as usize;

    println!("VTable Size: {:?}", page_size);
    println!("VTable Avg Size {:?}", avg_size);

    let doc = Document::dummy();

    let mut buffer = vtable.alloc_buf();

    let offsets = doc.to_docbuf(&mut buffer)?;

    println!("Offsets: {:?}", offsets);
    println!("Offsets Length: {:?}", offsets.len());
    println!("Offsets bytes length: {:?}", offsets.as_bytes().len());

    assert_eq!(
        offsets.as_bytes().len() / VTABLE_FIELD_OFFSET_SIZE_BYTES,
        vtable.num_offsets() as usize
    );
    println!("Buffer: {:?}", buffer.len());

    println!("Num page entries: {:?}", page_size % buffer.len());

    let avg_field_size = buffer.len() / (vtable.num_fields - vtable.num_items as u16) as usize;

    println!("Avg Field Size: {:?}", avg_field_size);

    Ok(())
}

#[cfg(feature = "db")]
#[test]
fn test_complex_db() -> Result<(), docbuf_db::Error> {
    use docbuf_db::traits::*;

    let doc = Document::dummy();

    let id = doc.db_insert()?;
    let partition_key = Some(doc.author.as_str());

    let doc = Document::db_get(id, partition_key)?;

    println!("Doc: {:?}", doc);

    Ok(())
}

#[test]
fn test_vtable_hash_tag() -> Result<(), docbuf_db::Error> {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let mut word_list = Vec::with_capacity(1e6 as usize);
    let word_dict = File::open("/usr/share/dict/words")?;
    let reader = BufReader::new(word_dict);
    for word in reader.lines() {
        word_list.push(word?);
    }

    let num_samples = word_list.len();
    let mut num_collisions = 0;

    let mut tag_counter = HashMap::<[u8; 2], usize>::with_capacity(100);
    let mut tags = HashMap::<[u8; 2], String>::with_capacity(100);

    for _ in 0..num_samples {
        let random_index = rand::random::<usize>() % word_list.len();
        let tag = word_list[random_index].clone();

        let hash = VTable::hash_tag(&tag);

        if let Some(tag2) = tags.get(&hash) {
            if tag != *tag2 {
                println!("Hash collision: {:?}", hash);
                println!("Tag 1: {:?}", tag);
                println!("Tag 2: {:?}\n", tag2);

                let count = tag_counter.entry(hash).or_insert(0);
                *count += 1;

                num_collisions += 1;
            }
        }

        tags.insert(hash, tag);
    }

    println!("Num Collisions: {:?}", num_collisions);
    println!("Num Samples: {:?}", num_samples);
    println!(
        "Collision Percentage: {}\n",
        num_collisions as f64 / num_samples as f64
    );

    let c1 = tag_counter.iter().filter(|(_, &v)| v == 1).count();
    let c2 = tag_counter.iter().filter(|(_, &v)| v == 2).count();
    let c3 = tag_counter.iter().filter(|(_, &v)| v == 3).count();
    let c4 = tag_counter.iter().filter(|(_, &v)| v == 4).count();
    let c5 = tag_counter.iter().filter(|(_, &v)| v == 5).count();
    let c5p = tag_counter.iter().filter(|(_, &v)| v > 5).count();

    println!("Found Tag with 1 Conflict: {:?}", c1);
    println!("Found Tag with 2 Conflict: {:?}", c2);
    println!("Found Tag with 3 Conflict: {:?}", c3);
    println!("Found Tag with 4 Conflict: {:?}", c4);
    println!("Found Tag with 5 Conflict: {:?}", c5);
    println!("Found Tag with >5 Conflicts: {:?}\n", c5p);

    // Find most conficting tag
    let mut max_conflict = 0;
    let mut max_conflict_tag = [0u8; 2];
    for (tag, count) in tag_counter.iter() {
        if *count > max_conflict {
            max_conflict = *count;
            max_conflict_tag = *tag;
        }
    }

    println!("Max Conflict: {:?}", max_conflict);
    println!("Max Conflict Tag: {:?}\n\n", max_conflict_tag);

    // Find Empty Tags
    let mut empty_tags = 0;
    for i in 0..255 {
        for j in 0..255 {
            let tag = [i, j];
            if !tags.contains_key(&tag) {
                // println!("Empty Tag: {:?}", tag);
                empty_tags += 1;
            }
        }
    }

    println!("Empty Tags: {:?}", empty_tags);
    println!(
        "Percentage Address Space Used: {:?}",
        1. - (empty_tags as f64 / u16::MAX as f64)
    );

    Ok(())
}

#[test]
fn test_complex_vtable() -> Result<(), docbuf_core::error::Error> {
    let vtable = Document::vtable()?;

    println!("VTable: {:?}", vtable);

    Ok(())
}

#[test]
fn test_partition_key() -> Result<(), docbuf_core::error::Error> {
    let num_observations = 1e6 as usize;

    let mut buckets = HashMap::new();

    let doc = Document::dummy();

    let doc_partition_key = doc.partition_key().expect("Failed to get partition key");
    let doc_bucket = doc_partition_key.bucket(None);
    println!("Partition Key: {:?}", doc_bucket);

    let entries = buckets.entry(doc_bucket).or_insert(0);
    *entries += 1;

    for _ in 0..num_observations {
        let partition_key = PartitionKey::from(rand::random::<u128>());
        let bucket = partition_key.bucket(None);

        let entries = buckets.entry(bucket).or_insert(0);
        *entries += 1;
    }

    let first_ten = buckets.iter().take(10);

    println!("Buckets: {:?}", first_ten.collect::<Vec<_>>());
    println!("Num Buckets: {:?}", buckets.len());
    println!("Dummy Doc Bucket: {:?}", buckets.get(&doc_bucket));

    Ok(())
}
