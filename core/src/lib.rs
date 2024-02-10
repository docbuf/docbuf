pub mod idl;
pub mod traits;
pub mod vtable;
pub mod serde;
pub mod error;

#[cfg(feature = "crypto")]
pub mod crypto {
    // Re-export the necessary crypto libraries for signing
    pub use digest;
    pub use ed25519;
    pub use sha2;
}

// Result type for the docbuf serialization crate
pub type Result<T> = std::result::Result<T, error::Error>;


