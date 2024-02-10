pub mod compiler;
pub mod document;
pub mod error;
pub mod lexer;
pub mod parser;

#[derive(Debug, Clone)]
pub enum Pragma {
    V1,
}