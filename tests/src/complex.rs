use docbuf_core::traits::DocBuf;
use docbuf_macros::*;
use serde::{Deserialize, Serialize};

use crate::{SetTestValues, TestHarness};

use docbuf_core::crypto::sha2;
use docbuf_core::traits::DocBufCrypto;

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
        max_length = 100;
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
    // #[docbuf {
    //     min_length = 0;
    // }]
    pub metadata: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf {
    sign = "true";
}]
pub struct Signature {
    #[docbuf {
        length = 32;
    }]
    pub signature: String,
    pub u8_data: u8,
}

impl Document {
    pub fn dummy() -> Self {
        Self {
            title: String::from("MyAwesomeDocument"),
            body: String::from("MyAwesomeDocument Contents"),
            footer: String::from("MyAwesomeDocument Copyright"),
            metadata: Metadata {
                metadata: String::from("MyAwesomeDocument Metadata"),
                signature: Signature {
                    signature: ["12345678"; 4].concat(),
                    u8_data: 0x0A,
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

    println!("document: {:?}", document);

    // let mut hasher = sha2::Sha256::default();
    // let hash = document.hash(&mut hasher)?;
    // println!("hash: {:?}", hash);

    document
        .assert_serialization_size()
        .expect("Failed encoding benchmark");

    Ok(())
}
