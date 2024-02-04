use std::collections::HashMap;

pub type FieldName = String;
pub type FieldMap = HashMap<FieldName, FieldOptions>;

#[derive(Debug, Clone, Default)]
pub enum FieldType {
    #[default]
    String,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Document(String),
    Enumerate(String),
    Array(Vec<FieldType>),
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Document(DocumentName),
    Enumerate(EnumerateName),
    Array(Vec<FieldValue>),
    // Raw string value, which has not been parsed to the field type
    Raw(String),
}

#[derive(Debug, Clone, Default)]
pub struct FieldOptions {
    pub comments: Option<String>,
    pub r#type: FieldType,
    pub name: Option<String>,
    pub required: Option<bool>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub min_value: Option<FieldValue>,
    pub max_value: Option<FieldValue>,
    pub regex: Option<String>,
    pub default: Option<FieldValue>,
}

pub type DocumentName = String;
pub type DocumentMap = HashMap<DocumentName, Document>;

#[derive(Clone, Debug, Default)]
pub struct Document {
    pub name: DocumentName,
    pub options: DocumentOptions,
    pub fields: FieldMap,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentOptions {
    // override the document name
    pub name: Option<String>,
    // denote if the document is a root document
    pub root: Option<bool>,
}

pub type EnumerateName = String;
pub type EnumMap = HashMap<EnumerateName, Enumerable>;

#[derive(Clone, Debug, Default)]
pub struct Enumerable {
    pub name: EnumerateName,
    pub options: EnumOptions,
    pub fields: FieldMap,
}

#[derive(Clone, Debug, Default)]
pub struct EnumOptions {
    pub comments: Option<String>,
    pub name: Option<String>,
}
