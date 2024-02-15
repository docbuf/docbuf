use docbuf_core::crypto::sha2;
use docbuf_core::traits::{DocBuf, DocBufCrypto};
use docbuf_macros::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize)]
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

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize)]
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

#[derive(Debug, Clone, DocBuf, Serialize, Deserialize)]
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

#[test]
fn test_docbuf_macros() -> Result<(), docbuf_core::error::Error> {
    let document = Document {
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
    };

    // let docbuf1 = document.to_docbuf()?;
    // println!("\n\ndocbuf1: {:?}", docbuf1);

    // let docbuf2 = document.to_docbuf()?;
    // println!("\n\ndocbuf2: {:?}", docbuf2);

    // assert_eq!(docbuf1, docbuf2);

    println!("document: {:?}", document);
    // let vtable = Document::vtable()?;
    // println!("vtable: {:#?}", vtable);

    let mut hasher = sha2::Sha256::default();
    let hash = document.hash(&mut hasher)?;
    println!("hash: {:?}", hash);

    let bytes = document.to_docbuf()?;

    println!("\n\ndocbuf bytes: {:?}", bytes);
    println!("docbuf bytes length: {:?}", bytes.len());

    let bincode_bytes = bincode::serialize(&document).unwrap();

    println!("\n\nbincode_bytes: {:?}", bincode_bytes);
    println!("bincode_bytes length: {:?}", bincode_bytes.len());

    assert!(bytes.len() <= bincode_bytes.len());

    let json_bytes = serde_json::to_string(&document).unwrap();
    let json_bytes = json_bytes.as_bytes();

    println!("\n\njson_bytes: {:?}", json_bytes);
    println!("json_bytes length: {:?}", json_bytes.len());

    assert!(bytes.len() <= json_bytes.len());

    let doc = Document::from_docbuf(&bytes)?;

    println!("doc: {:?}", doc);

    Ok(())
}
