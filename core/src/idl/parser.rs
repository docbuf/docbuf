use std::path::Path;

use crate::idl::{
    document::*,
    error::Error,
    lexer::{Logos, Token},
    Pragma,
};

#[derive(Debug, Clone)]
pub enum OptionSet {
    Document(DocumentOptions),
    Field(FieldOptions),
    Enum(EnumOptions),
    None,
}

#[derive(Debug, Clone)]
pub struct ParserContext {
    pub previous_span: std::ops::Range<usize>,
    pub previous_token: Option<Token>,
    pub current_document: Option<Document>,
    pub current_option_item: Option<String>,
    pub current_comments: Option<String>,
    pub current_option_set: OptionSet,
    pub current_field_name: Option<String>,
    pub current_enumerable: Option<Enumerable>,
}

impl ParserContext {
    pub fn new() -> ParserContext {
        ParserContext {
            previous_span: 0..0,
            previous_token: None,
            current_document: None,
            current_option_item: None,
            current_comments: None,
            current_option_set: OptionSet::None,
            current_field_name: None,
            current_enumerable: None,
        }
    }

    pub fn set_previous_token(&mut self, token: Token) {
        self.previous_token = Some(token);
    }

    pub fn set_previous_span(&mut self, span: std::ops::Range<usize>) {
        self.previous_span = span;
    }

    pub fn set_current_document(&mut self, document: Document) {
        self.current_document = Some(document);
    }

    pub fn set_current_option_item(&mut self, option_item: String) {
        self.current_option_item = Some(option_item);
    }

    pub fn set_current_comments(&mut self, comments: String) {
        if let Some(current_comments) = &mut self.current_comments {
            current_comments.push_str(&comments);
        } else {
            self.current_comments = Some(comments);
        }
    }

    pub fn set_current_option_set(&mut self, option_set: OptionSet) {
        self.current_option_set = option_set;
    }

    pub fn set_current_field_name(&mut self, field_name: String) {
        self.current_field_name = Some(field_name);
    }

    pub fn reset_current_option_item(&mut self) {
        self.current_option_item = None;
    }

    pub fn reset_current_option_set(&mut self) {
        self.current_option_set = OptionSet::None;
    }

    pub fn reset_current_field_name(&mut self) {
        self.current_field_name = None;
    }

    pub fn reset_current_comments(&mut self) {
        self.current_comments = None;
    }

    pub fn reset_current_document(&mut self) {
        self.current_document = None;
    }

    pub fn reset(&mut self) {
        self.reset_current_option_item();
        self.reset_current_option_set();
        self.reset_current_field_name();
        self.reset_current_comments();
        self.reset_current_document();
    }
}

#[derive(Debug, Clone)]
pub struct Parser {
    pub found_module_name: bool,
    pub context: ParserContext,
    pub pragma: Pragma,
    pub imports: Vec<Parser>,
    pub module_name: String,
    pub documents: DocumentMap,
    pub enumerates: EnumMap,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            found_module_name: false,
            context: ParserContext::new(),
            pragma: Pragma::V1,
            imports: Vec::new(),
            module_name: String::new(),
            documents: DocumentMap::default(),
            enumerates: EnumMap::default(),
        }
    }

    pub fn from_str(source: &str) -> Result<Self, Error> {
        let mut parser = Parser::new();

        let mut tokens = Token::lexer(source);

        parser.context.previous_token = tokens
            .next()
            .transpose()
            .map_err(|_| Error::Token(String::from("No Token Found.")))?;

        parser.context.previous_span = tokens.span();

        while let Some(token) = tokens.next() {
            if let Ok(token) = token {
                println!("Parsing Current Token: {:?}", token);

                match token {
                    Token::Pragma => {}
                    Token::Module => {
                        let span = tokens.span();
                        let is_mod_keyword = source[span.start..span.end + 1].ends_with(" ");

                        if is_mod_keyword {
                            // Do not allow multiple module names
                            if parser.found_module_name {
                                return Err(Error::InvalidModule);
                            }

                            // Mark the module found
                            parser.found_module_name = true;
                        } else {
                            // Exit early if the module name is not found
                            continue;
                        }
                    }
                    Token::Import => {
                        println!("Import: {:?}", tokens.slice());
                    }
                    Token::StatementEnd => {
                        match parser.context.previous_token {
                            Some(Token::Pragma) => {
                                let span =
                                    parser.context.previous_span.end + 1..tokens.span().start;
                                parser.pragma = Self::pragma(&source[span])?;
                            }
                            Some(Token::Module) => {
                                println!("Module: {:?}", tokens.slice());
                                let span =
                                    parser.context.previous_span.end + 1..tokens.span().start;
                                parser.module_name = source[span].trim().to_string();
                            }
                            Some(Token::Import) => {
                                let span =
                                    parser.context.previous_span.end + 1..tokens.span().start;
                                let import = source[span].trim().to_string();
                                println!("Import: {:?}", import);
                                // let import = Self::search_imports(source, Path::new("."))?;
                                // parser.imports = import;
                            }
                            Some(Token::StatementAssign) => {
                                if let Some(option_item) = &parser.context.current_option_item {
                                    let span =
                                        parser.context.previous_span.end..tokens.span().start;
                                    let option_value = source[span].trim().to_string();
                                    let value = option_value.trim().replace('\"', "");

                                    println!("Option Item: {:?}", option_item);
                                    println!("Option Value: {:?}", value);

                                    match &mut parser.context.current_option_set {
                                        OptionSet::Document(options) => {
                                            match option_item.as_str() {
                                                "name" => {
                                                    options.name = Some(value);
                                                }
                                                "root" => {
                                                    options.root = Some(value == "true");
                                                }
                                                _ => {}
                                            }

                                            if let Some(document) =
                                                &mut parser.context.current_document
                                            {
                                                document.options = options.clone();
                                            }
                                        }
                                        OptionSet::Field(options) => match option_item.as_str() {
                                            "min_length" => options.min_length = value.parse().ok(),
                                            "max_length" => options.max_length = value.parse().ok(),
                                            "min_value" => {
                                                options.min_value =
                                                    Some(FieldValue::Raw(value.clone()))
                                            }
                                            "max_value" => {
                                                options.max_value =
                                                    Some(FieldValue::Raw(value.clone()))
                                            }
                                            "regex" => options.regex = Some(value),
                                            "default" => {
                                                options.default =
                                                    Some(FieldValue::Raw(value.clone()))
                                            }
                                            "required" => {
                                                options.required = Some(value == "true");
                                            }
                                            "name" => {
                                                options.name = Some(value);
                                            }
                                            _ => {}
                                        },
                                        OptionSet::Enum(options) => match option_item.as_str() {
                                            "name" => {
                                                options.name = Some(value);
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }

                        println!("Parser: {:?}", parser);
                    }
                    Token::DocumentOptionsStart => {
                        // Set the current document to defaults
                        parser.context.set_current_document(Document::default());

                        // Set the current option set to document options
                        parser.context.set_current_option_set(OptionSet::Document(
                            DocumentOptions::default(),
                        ));
                    }
                    Token::CommentLine => {}
                    Token::NewLine => match parser.context.previous_token {
                        Some(Token::CommentLine) => {
                            let span = parser.context.previous_span.end + 1..tokens.span().start;
                            let comment = source[span].trim().to_string();

                            parser.context.set_current_comments(comment);
                        }
                        _ => {}
                    },
                    // Token::Name => {
                    //     let span = previous_span.end + 1..tokens.span().start;
                    //     let name = source[span].trim().to_string();

                    //     println!("Name: {:?}", name);

                    //     match previous_token {
                    //         Some(Token::Document) => {
                    //             if let Some(document) = &mut current_document {
                    //                 document.name = name;
                    //             }

                    //             println!("Current Document: {:?}", current_document);
                    //         }
                    //         _ => {}
                    //     }
                    // }
                    Token::StatementAssign => {
                        let span = parser.context.previous_span.end + 1..tokens.span().start;
                        let option_item = source[span].trim().to_string();

                        // Set the current option to be used by the next statement
                        parser.context.set_current_option_item(option_item);
                    }
                    Token::OptionsEnd => {
                        println!("Options Set: {:?}", parser.context.current_option_set);

                        // Reset the current option item
                        // current_option_item = None;
                        // current_option_set = OptionSet::None;

                        // unimplemented!("Options End")

                        // println!("Current Document: {:?}", current_document);

                        // if let Some(document) = &mut current_document {
                        //     let span = previous_span.start..tokens.span().end;
                        //     let options = Parser::document_options(&source[span])?;
                        //     document.options = options;
                        // }
                    }
                    Token::Document => {
                        println!("Current Document: {:?}", parser.context.current_document);
                    }
                    Token::Enumerate => {
                        let options = match parser.context.current_option_set.clone() {
                            OptionSet::Enum(options) => options,
                            _ => EnumOptions::default(),
                        };

                        parser.context.current_enumerable = Some(Enumerable {
                            options,
                            ..Default::default()
                        });
                    }
                    Token::SectionStart => {
                        let span = parser.context.previous_span.end + 1..tokens.span().start - 1;

                        match parser.context.previous_token {
                            Some(Token::Document | Token::OptionsEnd) => {
                                let document_name = source[span].trim().to_string();
                                println!("Document Name: {:?}", document_name);
                                if let Some(document) = &mut parser.context.current_document {
                                    document.name = document_name;
                                }
                            }
                            Some(Token::Enumerate) => {
                                let enumerate_name = source[span].trim().to_string();
                                println!("Enumerate Name: {:?}", enumerate_name);
                                if let Some(enumerable) = &mut parser.context.current_enumerable {
                                    enumerable.name = enumerate_name;
                                }
                            }
                            _ => {}
                        }
                    }
                    Token::SectionEnd => {
                        if let Some(document) = &parser.context.current_document {
                            parser
                                .documents
                                .insert(document.name.clone(), document.clone());
                        }

                        // Reset the context, preparing for the next section.
                        parser.context.reset();
                    }
                    Token::FieldOptionsStart => {
                        // Set the current option set to the field options
                        parser
                            .context
                            .set_current_option_set(OptionSet::Field(FieldOptions::default()));
                    }
                    Token::FieldDelimiter => match parser.context.previous_token {
                        Some(Token::OptionsEnd | Token::FieldEnd | Token::NewLine) => {
                            let span = parser.context.previous_span.end + 1..tokens.span().start;
                            let field_name = source[span].trim().to_string();
                            parser.context.set_current_field_name(field_name);
                        }
                        _ => {}
                    },
                    Token::FieldEnd => {
                        if let Some(field_name) = &parser.context.current_field_name {
                            let span = parser.context.previous_span.end + 1..tokens.span().start;
                            let field_value = source[span].trim().to_string();

                            println!("Field Name: {:?}", field_name);
                            println!("Field Value: {:?}", field_value);

                            if let Some(document) = &mut parser.context.current_document {
                                let options = match &mut parser.context.current_option_set {
                                    OptionSet::Field(options) => options.clone(),
                                    _ => FieldOptions::default(),
                                };

                                document.fields.insert(field_name.clone(), options);

                                println!("Current Document: {:?}", document);
                            }
                        } else {
                            unimplemented!("Field End")
                        }

                        parser.context.reset_current_field_name();
                        parser.context.reset_current_option_set();
                    }
                    Token::EnumOptionsStart => {
                        parser
                            .context
                            .set_current_option_set(OptionSet::Enum(EnumOptions::default()));
                    }
                    _ => {}
                }

                // Update the current token and span
                parser.context.previous_token = Some(token);
                parser.context.previous_span = tokens.span();
            }
        }

        // Throw an error if the module name does not exist.
        if !parser.found_module_name {
            return Err(Error::InvalidModule);
        }

        // println!("Tokens: {:?}", tokens);
        // let pragma = Self::pragma(source)?;
        // println!("Pragma: {:?}", pragma);
        // let document_options = Self::document_options(source)?;
        // println!("Document Options: {:?}", document_options);
        // // let documents = Self::documents(&file)?;
        // // println!("Documents: {:?}", documents);
        // parser.pragma = pragma;

        Ok(parser)
    }

    pub fn from_file(file: &Path) -> Result<Self, Error> {
        let parent_dir = file.parent().ok_or(Error::InvalidPath)?;

        println!("Parent Dir: {:?}", parent_dir);

        let file = std::fs::read_to_string(file)?;

        let mut parser = Self::from_str(&file)?;

        let imports = Self::search_imports(&file, parent_dir)?;

        parser.imports = imports;

        Ok(parser)
    }

    fn pragma(source: &str) -> Result<Pragma, Error> {
        println!("Source: {:?}", source);

        match source.trim() {
            "v1" => Ok(Pragma::V1),
            _ => Err(Error::MissingPragma),
        }
    }

    fn search_imports(source: &str, dir: &Path) -> Result<Vec<Parser>, Error> {
        // Search for the import statement, `import /path/to/file;` syntax
        let re = regex::Regex::new(r#"import (?P<file>.*);"#).unwrap();

        let files = re
            .captures_iter(source)
            .map(|cap| cap["file"].trim().replace('\"', ""))
            .collect::<Vec<String>>();

        let files = files
            .iter()
            .map(|file| {
                let file = dir.join(file);
                println!("File: {:?}", file);
                Self::from_file(&file).unwrap()
            })
            .collect::<Vec<Parser>>();

        Ok(files)
    }

    // fn documents(source: &str) -> Result<(), Error> {
    //     // Search for the document type, `Document MyDocumentType { ... }` syntax

    //     let re = regex::Regex::new(r#"Document (?<name>\w+) {(?<fields>\w+)}"#).unwrap();

    //     for cap in re.captures_iter(source) {
    //         let name = &cap["name"];
    //         let fields = &cap["fields"];

    //         println!("Name: {}", name);
    //         println!("Fields: {}", fields);
    //     }

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_parser() -> Result<(), Box<dyn std::error::Error>> {
        let file = Path::new("../examples/example.docbuf");

        Parser::from_file(&file)?;

        Ok(())
    }

    #[test]
    fn test_token_stream() {
        use proc_macro2::TokenStream;

        let file =
            std::fs::read_to_string("../examples/example.docbuf").expect("Failed to read file");

        let tokens: TokenStream = file.parse().unwrap();

        println!("tokens: {:?}", tokens);
    }
}
