pub use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Search for the pragma version
    #[token("pragma docbuf")]
    Pragma,
    // Search for the module name
    #[token("\nmodule")]
    Module,
    // Search for imports
    #[token("\nimport")]
    Import,
    // Search for regex matching `document <name> {`
    #[token("\ndocument")]
    Document,
    // Search for enumerable items
    #[token("\nenumerable")]
    Enumerate,
    // Search for process statements
    #[token("\nprocess")]
    Process,
    // // Search for document, field, process names
    // #[regex("[a-zA-Z0-9]+")]
    // Name,
    // Search for option assignments
    #[token(" = ")]
    StatementAssign,
    // Search for the end of a statement
    #[token(";")]
    StatementEnd,
    // Search for document options
    #[token("#[document::options {")]
    DocumentOptionsStart,
    // Search for field options
    #[token("#[field::options {")]
    FieldOptionsStart,
    // Search for enum options
    #[token("#[enum::options {")]
    EnumOptionsStart,
    // Search for item options
    #[token("#[item::options {")]
    ItemOptionsStart,
    // Search for process options
    #[token("#[process::options {")]
    ProcessOptionsStart,
    // Search for the end of options
    #[token("\n}]")]
    OptionsEnd,
    // Search for the start of a section
    #[token("{")]
    SectionStart,
    // Search for the end of a section
    #[token("\n}")]
    SectionEnd,
    // Search for the field delimiter
    #[token(":")]
    FieldDelimiter,
    // Search for the end of the field
    #[token(",")]
    FieldEnd,
    // Search for a comment line
    #[token("// ")]
    CommentLine,
    // Search for a documentation comment line
    #[token("/// ")]
    DocCommentLine,
    // Search for a new line
    #[regex(r"\n", |lex| lex.bump(1))]
    NewLine,
}
