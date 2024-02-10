use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use proc_macro2::{Ident, TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{parse::Parser, DeriveInput, ItemStruct};

// TODO: Replace with OID
#[derive(Debug, Clone)]
pub(crate) enum DocBufCryptoAlgorithm {
    Ed25519,
}

// TODO: Replace with external library
#[derive(Debug, Clone)]
pub(crate) enum HashAlgorithm {
    Sha256,
}

#[derive(Debug, Clone)]
pub(crate) enum DocBufOpt {
    Sign(bool),
    Crypto(DocBufCryptoAlgorithm),
    Hash(HashAlgorithm),
}

#[derive(Debug, Clone)]
pub(crate) struct DocBufOpts(HashMap<String, DocBufOpt>);

impl From<TokenStream> for DocBufOpts {
    fn from(input: TokenStream) -> Self {
        println!("Parsing DocBufOpts: {:#?}", input);
        
        let mut iter = input.into_iter();
        let mut num_attr = iter.clone().count() / 4;
        let key_index = 0;
        let value_index = 1;
        let mut opts = HashMap::new();

        

        while num_attr > 0 {
            println!("Num Attr: {}", num_attr);
            println!("Key Index: {}", key_index);
            println!("Value Index: {}", value_index);

            let mut span_iter = {
                let mut span = Vec::new();

                for _ in 0..4 {
                    span.push(iter.next().unwrap());
                }

                span.into_iter()
            };

            println!("Span Iter: {:?}", span_iter);

            if let Some(key) = span_iter.nth(key_index) {
                if let Some(value) = span_iter.nth(value_index) {
                    let key = key.to_string().replace("\"", "");
                    let value = value.to_string().replace("\"", "");

                    println!("Key: {:#?}", key);
                    println!("Value: {:#?}", value);
                    
                    let value = match key.as_str() {
                        "sign" => DocBufOpt::Sign("true" == value.as_str()),
                        "crypto" => match value.as_str() {
                            "ed25519" => DocBufOpt::Crypto(DocBufCryptoAlgorithm::Ed25519),
                            _ => { panic!("`crypto` options currently only supports `ed25519` value type.") }
                        }
                        "hash" => match value.as_str() {
                            "sha256" => DocBufOpt::Hash(HashAlgorithm::Sha256),
                            _ => { panic!("`hash` options currently only supports `sha256` value type.") }
                        }
                        k => {
                            unimplemented!("Unsupported key: {}", k);
                        }
                    };
        
                    opts.insert(key, value);
                }
            }

            num_attr -= 1;
        }

        println!("DocBufOpts: {:#?}", opts);        

        Self(opts)
    }
}

// Parse the attributes of the derive macro
pub(crate) fn docbuf_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    
    // println!("item: {:#?}", item);

    println!("Parsing DocBuf Derive macro attributes");
    // println!("\n\nattr: {:#?}", attr);




    TokenStream::new()
}

pub(crate) fn docbuf_item(item: TokenStream) -> TokenStream {
    // println!("\n\nitem: {:#?}", item);

    let name = parse_item_name(&item);
    let fields = parse_item_fields(&item);

    let output = quote! {
        pub struct #name {
            #fields
        }
    };

    TokenStream::from(output)
}

pub(crate) fn docbuf_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);
    // // println!("Attr: {:#?}", attr);
    // let mut options = DocBufOpts::from(attr.clone());
    // println!("Options: {:#?}", options);
    // let crypto_methods = docbuf_impl_crypto(&name, &mut options);

    let serialization_methods = docbuf_impl_serialization(item.clone());
    let vtable = docbuf_impl_vtable(item.clone());
    
    let output = quote! {
        impl ::docbuf_core::traits::DocBuf for #name {
            type Doc = Self;
            type DocBuf = Self;

            fn to_doc(self) -> Self::Doc {
                self
            }

            fn as_doc(&self) -> &Self::Doc {
                self
            }

            fn from_doc(doc: Self::Doc) -> Self {
                doc
            }

            // Serialization Methods
            #serialization_methods

            // VTable
            #vtable
        }
    };

    TokenStream::from(output)
}

// Impl docbuf signing for the input struct
pub(crate) fn docbuf_impl_crypto(name: &TokenStream, options: &mut DocBufOpts) -> TokenStream {
    let mut output = Vec::new();
    
    // Check if the sign option is present
    if let Some(DocBufOpt::Sign(true)) = options.0.get("sign") {
        println!("Sign Option Present");

        output.push(quote! {
            impl ::docbuf_core::traits::DocBufCrypto for #name { }
        });
    }

    TokenStream::from(quote! {
        #(#output)*
    })
}

pub(crate) fn docbuf_impl_vtable(item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);

    let ast: ItemStruct = syn::parse(item.to_owned().into()).unwrap();
    
    let fields = ast.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let ty = field.ty.to_token_stream();
        // let vis = &field.vis;
    
        println!("Field name: {:#?}", name);

        println!("Field type: {:#?}", ty);

        match ::docbuf_core::vtable::FieldType::is_struct(ty.to_string().as_ref()) {
            true => {
                let table_name = format!("{}_vtable", ty.to_string()).to_lowercase();
                let table_name_var = Ident::new(&table_name, Span::call_site());

                let struct_name = format!("{}_struct", ty.to_string()).to_lowercase();
                let struct_name_var = Ident::new(&struct_name, Span::call_site());

                let scope = quote! {
                    // Lookup the vtable for the struct
                    let #table_name_var = #ty::vtable()?;
                    
                    if let Some(#struct_name_var) = #table_name_var.structs.get(stringify!(#ty).as_bytes()) {
                        let field_type = ::docbuf_core::vtable::FieldType::Struct(#struct_name_var.clone().struct_name_as_bytes);
                        
                        vtable_struct.add_field(field_type, stringify!(#name));
                    }

                    // Merge the vtable with the input vtable
                    vtable.merge_vtable(#table_name_var);
                };

                println!("\n\nScope: {:#?}\n\n", scope.to_string());

                scope
            },
            false => {
                quote! {
                    vtable_struct.add_field(stringify!(#ty), stringify!(#name));
                }
            }
        }        
    });

    quote! {
        fn vtable() -> Result<::docbuf_core::vtable::VTable, ::docbuf_core::error::Error> {
            let mut vtable =  ::docbuf_core::vtable::VTable::new();

            let mut vtable_struct = ::docbuf_core::vtable::VTableStruct::new(stringify!(#name), None);

            // Add the fields to the vtable
            #(#fields)*

            // Create a vtable_struct for the input struct
            vtable.add_struct(vtable_struct);

            Ok(vtable)
        }
    }
}

// Impl docbuf serialization and deserialization for the input struct
pub(crate) fn docbuf_impl_serialization(input: TokenStream) -> TokenStream {
    let output = quote! {
        // Serialize the struct to a byte buffer
        fn to_docbuf(&self) -> Result<Vec<u8>, ::docbuf_core::error::Error> {
            Ok(::docbuf_core::serde::ser::to_docbuf(self)?)
        }
    
        // Deserialize the byte buffer to a struct
        fn from_docbuf(buf: &[u8]) -> Result<Self, ::docbuf_core::error::Error> {
            unimplemented!("from_docbuf")
        }
    };

    TokenStream::from(output)
}

pub(crate) fn docbuf_required_macros() -> TokenStream {
    let macros = vec![
        "std::fmt::Debug", "Clone", "::serde::Serialize", "::serde::Deserialize"
    ].join(", ");

    let macros = TokenStream::from_str(&macros).unwrap();

    let output = quote! {
        #[derive(#macros)]
    };

    output
}

pub(crate) fn derive_docbuf(attr: TokenStream, item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);

    // println!("Attr: {:#?}", attr);

    let mut options = DocBufOpts::from(attr.clone());
    println!("Options: {:#?}", options);
    
    // println!("\n\nitem: {:#?}", item);
    // Required derive macros
    let derive_macros = docbuf_required_macros();

    // let attr_docbuf = docbuf_attr(attr.clone(), item.clone());
    // parse the inner field attributes of the item
    let item_docbuf = docbuf_item(item.clone());
    
    // Add crypto methods from the options
    let crypto_methods = docbuf_impl_crypto(&name, &mut options);


    // let ast: ItemStruct = syn::parse2(item.clone()).unwrap();

    let output = quote! {
        // #attr_docbuf
        // #derive_macros
        #item_docbuf

        #crypto_methods
    };

    println!("Output: {:?}", output.to_string());

    TokenStream::from(output)
}

// Parse the item name from the input token stream
pub(crate) fn parse_item_name(input: &TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.to_owned().into()).unwrap();
    ast.ident.to_token_stream()
}

// Parse the item fields from the input stream
pub(crate) fn parse_item_fields(input: &TokenStream) -> TokenStream {
    let ast: ItemStruct = syn::parse(input.to_owned().into()).unwrap();
    let fields = ast.fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let ty = field.ty.to_token_stream();
        let vis = &field.vis;
    
        quote! {
            #vis #name: #ty
        }
    });

    quote! {
        #(#fields),*
    }
}