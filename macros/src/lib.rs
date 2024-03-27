use docbuf_derive::{derive, proc_macro2, quote::quote};

use proc_macro::TokenStream;

#[proc_macro_derive(DocBuf)]
pub fn derive_docbuf(item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(TokenStream::new());
    let item = proc_macro2::TokenStream::from(item);

    let name = derive::parse_item_name(&item);
    let lifetimes = derive::parse_item_lifetimes(&item);
    let options = derive::DocBufOpts::from(&attr);

    let docbuf_methods = derive::docbuf_impl(&name, &lifetimes, &options, &item);
    let crypto_methods = derive::docbuf_impl_crypto(&name, &lifetimes, &options);
    let db_methods = derive::docbuf_impl_db(&name, &lifetimes, &options, &item);

    quote! {
        #docbuf_methods
        #crypto_methods
        #db_methods
    }
    .into()
}

#[proc_macro_attribute]
pub fn docbuf(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    derive::derive_docbuf(attr, item).into()
}
