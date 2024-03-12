// #[cfg(feature = "db")]
// pub mod database;
pub mod error;
// #[cfg(feature = "macros")]
// pub mod macros;
pub mod serde;
pub mod traits;
pub mod vtable;

#[cfg(feature = "crypto")]
pub mod crypto {
    // Re-export the necessary crypto libraries for signing
    pub use digest;
    pub use ed25519;
    pub use sha2;
}

#[cfg(feature = "validate")]
pub mod validate {
    // Re-export the necessary validation libraries for validating fields
    pub use regex;
}

// Result type for the docbuf core crate
pub type Result<T> = std::result::Result<T, error::Error>;
