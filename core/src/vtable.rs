mod fields;
mod structs;
mod table;

pub use fields::*;
pub use structs::*;
pub use table::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Field length mismatch")]
    FieldLengthMismatch,
    #[error("Struct Not Found")]
    StructNotFound,
    #[error("Field Not Found")]
    FieldNotFound,
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Syn(#[from] syn::Error),
}

#[cfg(test)]
mod tests {}
