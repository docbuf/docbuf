pub mod compiler;
pub mod document;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod traits;
pub mod vtable;

pub use docbuf_serde as serde;

#[cfg(feature = "crypto")]
pub mod crypto {
    // Re-export the necessary crypto libraries for signing
    pub use digest;
    pub use ed25519;
    pub use sha2;
}

#[derive(Debug, Clone)]
pub enum Pragma {
    V1,
}
