mod bufmap;
mod field;
mod item;
mod table;

// pub use bufmap::*;
pub use field::*;
pub use item::*;
pub use table::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Field length mismatch")]
    FieldLengthMismatch,
    #[error("Field Rules Length: {0}")]
    FieldRulesLength(String),
    #[error("Item Not Found")]
    ItemNotFound,
    #[error("Struct Not Found")]
    StructNotFound,
    #[error("Field Not Found")]
    FieldNotFound,
    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Syn(#[from] syn::Error),
    #[cfg(feature = "regex")]
    #[error(transparent)]
    #[cfg(feature = "regex")]
    Regex(#[from] ::regex::Error),
    #[error("Field Rules Invalid Regex: {0}")]
    FieldRulesRegex(String),
    #[error("Field Rules Invalid Value: {0}")]
    FieldRulesValue(String),
    #[error("Invalid field type for validation: {0}")]
    InvalidValidationType(String),
    #[error("Unable to borrow mutable reference for vtable")]
    VTableBorrowMut,
    #[error("Unable to parse encoded data")]
    FailedToParseData,
    #[error("Map entries exceeds max: {0}")]
    MapEntriesExceedsMax(usize),
    #[error("Array length exceeds max: {0}")]
    ArrayElementsExceedsMax(usize),
    #[error("Invalid docbuf map field type, expected: {0}")]
    DocBufMapInvalidFieldType(String),
    #[error("Failed to encode docbuf map field type: {0}")]
    DocBufEncodeFieldType(String),
    #[error("Failed to decode docbuf map field type: {0}")]
    DocBufDecodeFieldType(String),
}

#[cfg(test)]
mod tests {}
