use docbuf_core::traits::{DocBuf, DocBufCrypto};
use docbuf_macros::*;
use serde::{Deserialize, Serialize};

use crate::{SetTestValues, TestHarness};

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

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf {
    sign = true;
}]
pub struct Metadata {
    #[docbuf {
        min_length = 0;
    }]
    pub description: String,
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
    pub signature: Signature,
}

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf {
    // sign = "true";
}]
pub struct Signature {
    #[docbuf {
        length = 32;
    }]
    pub signature: String,
}

impl Document {
    pub fn dummy() -> Self {
        Self {
            title: ["0"; 64].concat(),
            body: ["0"; 2048].concat(),
            footer: ["0"; 32].concat(),
            metadata: Metadata {
                description: ["0"; 512].concat(),
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
                signature: Signature {
                    signature: ["0"; 32].concat(),
                },
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

    // let mut hasher = sha2::Sha256::default();
    // let hash = document.hash(&mut hasher)?;
    // println!("hash: {:?}", hash);

    document
        // Round Trip Test
        .assert_serialization_round_trip(&mut buffer)
        .expect("Failed round trip serialization")
        // Serialization Size Comparison Test
        .assert_serialization_size(&mut buffer)
        .expect("Failed encoding benchmark");

    let doc = Document::from_docbuf(&mut buffer)?;

    println!("doc: {:?}", doc);

    Ok(())
}
