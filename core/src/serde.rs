pub mod de;
pub mod ser;

// Re-export serde dependencies
pub use serde_bytes;

#[cfg(test)]
mod tests {}
