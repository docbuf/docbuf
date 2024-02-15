use proc_macro::TokenStream;

//
use docbuf_core::macros::{derive, proc_macro2, quote::quote};

#[proc_macro_derive(DocBuf)]
pub fn derive_docbuf(input: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(TokenStream::new());
    let input = proc_macro2::TokenStream::from(input);

    let impl_docbuf = derive::docbuf_impl(attr.clone(), input.clone());

    quote! {
        #impl_docbuf
    }
    .into()
}

#[proc_macro_attribute]
pub fn docbuf(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    derive::derive_docbuf(attr.clone(), item.clone()).into()
}
