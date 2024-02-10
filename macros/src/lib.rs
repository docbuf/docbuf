mod derive;

use docbuf_core::lexer::Token;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(DocBuf)]
pub fn derive_docbuf(input: TokenStream) -> TokenStream {
    println!("Deriving DocBuf");

    let attr = proc_macro2::TokenStream::from(TokenStream::new());
    let input = proc_macro2::TokenStream::from(input);

    // derive::derive_docbuf(attr.clone(), input.clone()).into()

    println!("input: {:#?}", input);

    TokenStream::new()
}


#[proc_macro_attribute]
pub fn docbuf(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("Parsing DocBuf Derive macro"); 
    
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);
    
    derive::derive_docbuf(attr.clone(), item.clone()).into()
}

// #[proc_macro_attribute]
// pub fn docbuf_opt(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     println!("\nattr: {:?}", _attr);
//     println!("\nitem: {:?}", item);

//     item
// }