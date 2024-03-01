use docbuf_core::{
    traits::{DocBuf, DocBufCrypto, DocBufMap},
    vtable::VTableFieldOffset,
};
use docbuf_macros::*;
use serde::{Deserialize, Serialize};

use crate::{SetTestValues, TestHarness};

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
// #[docbuf {

// }]
pub struct Complex(Vec<Document>);

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf {
    // Sign the entire document, will create an allocation for the document
    // signature
    sign = true;
    // Use the ed25519 signature algorithm
    // crypto = "ed25519";
    // Use the sha256 hash algorithm
    // hash = "sha256";
    html = "path/to/html/template.html";
}]
pub struct Document {
    #[docbuf {
        // Ignore the title field
        ignore = true;
    }]
    pub title: String,
    #[docbuf {
        min_length = 0;
        max_length = 4096;
        default = "Hello, World!";
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

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf {
    sign = true;
}]
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
    pub byte_data: Vec<u8>,
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
    }
}

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf {
    // sign = "true";
}]
pub struct Signature {
    #[docbuf {
        length = 32;
    }]
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

    let offsets = document.to_docbuf(&mut buffer)?;

    println!("Buffer: {:?}\n\n", buffer);

    println!("Offsets: {:?}\n\n", offsets);

    let sig_field_data: Vec<u8> = Document::vtable()?.docbuf_map(&buffer, &offsets[4])?;

    println!("Field: {:?}\n\n", sig_field_data);

    assert_eq!(
        document.metadata.signature.signature.to_vec(),
        sig_field_data
    );

    Ok(())
}
