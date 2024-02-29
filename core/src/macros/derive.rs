use std::collections::HashMap;
use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{DeriveInput, ItemStruct};

use crate::error::Error;

// TODO: Replace with OID
#[derive(Debug, Clone)]
pub enum DocBufCryptoAlgorithm {
    Ed25519,
}

// TODO: Replace with external library
#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    Sha256,
}

#[derive(Debug, Clone)]
pub enum DocBufOpt {
    Sign(bool),
    Crypto(DocBufCryptoAlgorithm),
    Hash(HashAlgorithm),
    Html(bool),
}

#[derive(Debug, Clone)]
pub struct DocBufOpts(HashMap<String, DocBufOpt>);

impl From<TokenStream> for DocBufOpts {
    fn from(input: TokenStream) -> Self {
        let mut iter = input.into_iter();
        let mut num_attr = iter.clone().count() / 4;
        let key_index = 0;
        let value_index = 1;
        let mut opts = HashMap::new();

        while num_attr > 0 {
            let mut span_iter = {
                let mut span = Vec::new();

                for _ in 0..4 {
                    span.push(iter.next().unwrap());
                }

                span.into_iter()
            };

            if let Some(key) = span_iter.nth(key_index) {
                if let Some(value) = span_iter.nth(value_index) {
                    let key = key.to_string().replace("\"", "");
                    let value = value.to_string().replace("\"", "");

                    let value = match key.as_str() {
                        "sign" => DocBufOpt::Sign("true" == value.as_str()),
                        "crypto" => match value.as_str() {
                            "ed25519" => DocBufOpt::Crypto(DocBufCryptoAlgorithm::Ed25519),
                            _ => {
                                panic!("`crypto` options currently only supports `ed25519` value type.")
                            }
                        },
                        "hash" => {
                            match value.as_str() {
                                "sha256" => DocBufOpt::Hash(HashAlgorithm::Sha256),
                                _ => {
                                    panic!("`hash` options currently only supports `sha256` value type.")
                                }
                            }
                        }
                        "html" => DocBufOpt::Html("true" == value.as_str()),
                        k => {
                            unimplemented!("Unsupported key: {}", k);
                        }
                    };

                    opts.insert(key, value);
                }
            }

            num_attr -= 1;
        }

        Self(opts)
    }
}

// Parse the attributes of the derive macro
pub fn docbuf_attr(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    TokenStream::new()
}

pub fn docbuf_item(item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);
    let fields = parse_item_fields(&item);
    let lifetimes = parse_item_lifetimes(&item);

    let output = quote! {
        pub struct #name #lifetimes {
            #fields
        }
    };

    TokenStream::from(output)
}

pub fn docbuf_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);

    // let mut options = DocBufOpts::from(attr.clone());

    // let crypto_methods = docbuf_impl_crypto(&name, &mut options);

    let lifetimes = parse_item_lifetimes(&item);

    let serialization_methods = docbuf_impl_serialization(item.clone());
    let vtable = docbuf_impl_vtable(item.clone());

    let output = quote! {
        impl #lifetimes ::docbuf_core::traits::DocBuf for #name #lifetimes {
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
pub fn docbuf_impl_crypto(
    name: &TokenStream,
    item: &TokenStream,
    options: &mut DocBufOpts,
) -> TokenStream {
    let mut output = Vec::new();

    // Check if the sign option is present
    if let Some(DocBufOpt::Sign(true)) = options.0.get("sign") {
        let lifetimes = parse_item_lifetimes(item);

        output.push(quote! {
            impl #lifetimes ::docbuf_core::traits::DocBufCrypto for #name #lifetimes { }
        });
    }

    quote! {
        #(#output)*
    }
}

pub fn docbuf_impl_vtable(item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);

    let ast: ItemStruct = syn::parse(item.to_owned().into()).unwrap();

    let fields = ast.fields.into_iter().collect::<Vec<_>>();

    let fields = fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let ty = field.ty.to_token_stream();
        let rules = parse_field_rules(&field).expect("Failed to parse field rules");

        match crate::vtable::FieldType::is_struct(ty.to_string().as_ref()) {
            true => {
                let table_name = format!("{}_vtable", ty.to_string()).to_lowercase();
                let table_name_var = Ident::new(&table_name, Span::call_site());

                let struct_name = format!("{}_struct", ty.to_string()).to_lowercase();
                let struct_name_var = Ident::new(&struct_name, Span::call_site());

                let scope = quote! {
                    // Lookup the vtable for the struct
                    let #table_name_var = #ty::vtable().expect("Failed to lookup vtable for struct");

                    for vtable_item in #table_name_var.items.iter() {
                        match vtable_item {
                            ::docbuf_core::vtable::VTableItem::Struct(#struct_name_var) => {
                                let name = #struct_name_var.struct_name;
                                if name == stringify!(#ty) {
                                    let field_type = ::docbuf_core::vtable::FieldType::Struct(name);

                                    // Add the field rules to the vtable field
                                    #rules
                                    vtable_struct.add_field(field_type, stringify!(#name), field_rules);
                                }
                            }
                            _ => {
                                unimplemented!("Unimeplemented vtable item type");
                            }
                        }
                    }

                    // Merge the vtable with the input vtable
                    vtable.merge_vtable(#table_name_var);
                };



                scope
            },
            false => {
                quote! {
                    // Add the field rules to the vtable field
                    #rules

                    vtable_struct.add_field(stringify!(#ty), stringify!(#name), field_rules);
                }
            }
        }
    });

    let vtable = quote! {
        fn vtable() -> Result<&'static ::docbuf_core::vtable::VTable<'static>, ::docbuf_core::error::Error> {
            static VTABLE: ::std::sync::OnceLock<::docbuf_core::vtable::VTable> = ::std::sync::OnceLock::new();

            let vtable = VTABLE.get_or_init(||  {
                let mut vtable = ::docbuf_core::vtable::VTable::new();

                let mut vtable_struct = ::docbuf_core::vtable::VTableStruct::new(stringify!(#name), None);

                // Add the fields to the vtable
                #(#fields)*

                // Sorting is required to ensure the structs are added in a consistent order
                vtable.items.inner_mut().sort_by(|a, b| match (a, b) {
                    (::docbuf_core::vtable::VTableItem::Struct(a), ::docbuf_core::vtable::VTableItem::Struct(b)) => a.struct_name
                        .cmp(&b.struct_name)
                        .then(a.item_index.cmp(&b.item_index)),
                    _ => std::cmp::Ordering::Equal,
                });

                // Create a vtable_struct for the input struct
                vtable.add_struct(vtable_struct);

                vtable
            });

            Ok(vtable)
        }
    };

    vtable
}

// Impl docbuf serialization and deserialization for the input struct
pub fn docbuf_impl_serialization(_input: TokenStream) -> TokenStream {
    let output = quote! {
        // Serialize the struct to a byte buffer
        fn to_docbuf<'a>(&self, buffer: &'a mut Vec<u8>) -> Result<(), ::docbuf_core::error::Error> {
            ::docbuf_core::serde::ser::to_docbuf(self, buffer)?;

            Ok(())
        }

        // Deserialize the byte buffer to a struct
        fn from_docbuf(buf: &[u8]) -> Result<Self, ::docbuf_core::error::Error> {
            Ok(::docbuf_core::serde::de::from_docbuf(buf)?)
        }
    };

    TokenStream::from(output)
}

pub fn docbuf_required_macros() -> TokenStream {
    let macros = vec![
        "std::fmt::Debug",
        "Clone",
        "::serde::Serialize",
        "::serde::Deserialize",
    ]
    .join(", ");

    let macros = TokenStream::from_str(&macros).unwrap();

    let output = quote! {
        #[derive(#macros)]
    };

    output
}

pub fn derive_docbuf(attr: TokenStream, item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);
    let mut options = DocBufOpts::from(attr.clone());

    // Required derive macros
    // let derive_macros = docbuf_required_macros();

    // let attr_docbuf = docbuf_attr(attr.clone(), item.clone());
    // parse the inner field attributes of the item
    let item_docbuf = docbuf_item(item.clone());

    // Add crypto methods from the options
    let crypto_methods = docbuf_impl_crypto(&name, &item, &mut options);

    let output = quote! {
        // #attr_docbuf
        // #derive_macros
        #item_docbuf

        #crypto_methods
    };

    TokenStream::from(output)
}

// Parse the item name from the input token stream
pub fn parse_item_name(input: &TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.to_owned().into()).unwrap();
    ast.ident.to_token_stream()
}

// Parse the item fields from the input stream
pub fn parse_item_fields(input: &TokenStream) -> TokenStream {
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

pub fn parse_item_lifetimes(input: &TokenStream) -> TokenStream {
    let ast: ItemStruct = syn::parse(input.to_owned().into()).unwrap();

    let lifetimes = ast.generics.lifetimes();
    let count = lifetimes.count();

    if count == 0 {
        TokenStream::new()
    } else {
        let lifetimes = ast.generics.lifetimes().map(|lifetime| {
            let lifetime = lifetime.to_token_stream();

            quote! {
                #lifetime
            }
        });

        quote! {
            <#(#lifetimes),*>
        }
    }
}

// Parses the input and creates a token stream from the input
// for constructing the field rules from the attributes input
pub fn parse_field_rules(input: &syn::Field) -> Result<TokenStream, Error> {
    let fields = input
        .attrs
        .iter()
        .map(|attr| {
            let field_type = &input.ty;

            let attributes = parse_docbuf_field_attrs(attr.to_token_stream())?;

            let rules = attributes
                .iter()
                .map(|(_, (key, value))| match key.to_string().as_str() {
                    "sign" | "ignore" => {
                        quote! {
                            field_rules.#key = #value;
                        }
                    }
                    "min_value" | "max_value" => {
                        quote! {
                            field_rules.#key = Some((#value as #field_type).into());
                        }
                    }
                    "min_length" | "max_length" | "length" => {
                        quote! {
                            field_rules.#key = Some(#value);
                        }
                    }
                    #[cfg(feature = "regex")]
                    "regex" => {
                        quote! {
                            field_rules.#key = Some(
                                ::docbuf_core::validate::regex::Regex::new(#value)
                                    .expect(&format!("Invalid regex: {:?}", #value))
                            );
                        }
                    }
                    _ => {
                        quote!()
                    }
                })
                .collect::<Vec<TokenStream>>();

            Ok(quote!(
                #(#rules)*
            ))
        })
        .collect::<Result<Vec<TokenStream>, Error>>()?;

    let rules = quote!(
        let mut field_rules = ::docbuf_core::vtable::FieldRules::new();

        #(#fields)*
    );

    Ok(rules)
}

pub fn parse_docbuf_field_attrs(
    input: TokenStream,
) -> Result<HashMap<String, (TokenTree, TokenTree)>, Error> {
    let mut map = HashMap::new();

    for attribute in input.into_iter() {
        match attribute {
            TokenTree::Group(group) => {
                let tokens = group.stream();
                for token in tokens.into_iter() {
                    match token {
                        TokenTree::Group(group) => {
                            let group_tokens = group.stream();
                            let mut key = None;
                            let mut value = None;

                            for group_token in group_tokens.into_iter() {
                                match &group_token {
                                    TokenTree::Ident(ident) => match ident.to_string().as_str() {
                                        "true" | "false" => {
                                            value = Some(group_token);
                                        }
                                        _ => {
                                            key = Some(group_token);
                                        }
                                    },
                                    TokenTree::Literal(_) => {
                                        value = Some(group_token);
                                    }
                                    _ => {}
                                }

                                if let (Some(key), Some(value)) = (key.clone(), value.clone()) {
                                    map.insert(key.to_string(), (key, value));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(map)
}
