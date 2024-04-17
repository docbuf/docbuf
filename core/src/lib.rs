pub mod error;
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

#[cfg(feature = "db")]
pub mod db {
    // Re-export db utilities
}

// Re-export the necessary deps for the docbuf core crate
pub mod deps {
    pub use hex;

    #[cfg(feature = "uuid")]
    pub use uuid;
}

// Result type for the docbuf core crate
pub type Result<T> = std::result::Result<T, error::Error>;
