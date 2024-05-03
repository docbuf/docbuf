use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::ItemImpl;

pub fn parse_impl(input: &TokenStream) -> TokenStream {
    let implementation: ItemImpl =
        syn::parse(input.to_owned().into()).expect("Failed to parse input");

    println!("Item {:?}", implementation.self_ty.to_token_stream());

    for item in implementation.items.iter() {
        println!("Item {:?}", item.to_token_stream());

        match item {
            syn::ImplItem::Fn(function) => {
                println!("Function {:?}", function.sig.ident.to_token_stream());

                for arg in function.sig.inputs.iter() {
                    println!("Arg {:?}", arg.to_token_stream());
                }
            }
            _ => {}
        }
    }

    unimplemented!("Not implemented: parse_impl")
}
