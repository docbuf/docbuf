pub mod de;
pub mod ser;
pub mod error;


// Result type for the docbuf serialization crate
pub type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod tests {
    
}
