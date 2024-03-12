use docbuf_derive::{derive, proc_macro2, quote::quote};

use proc_macro::TokenStream;

#[proc_macro_derive(DocBuf)]
pub fn derive_docbuf(_input: TokenStream) -> TokenStream {
    // let attr = proc_macro2::TokenStream::from(TokenStream::new());
    // let input = proc_macro2::TokenStream::from(input);

    // let impl_docbuf = derive::docbuf_impl(attr, input);

    quote! {
        // #impl_docbuf
    }
    .into()
}

#[proc_macro_attribute]
pub fn docbuf(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    derive::derive_docbuf(attr, item).into()
}
