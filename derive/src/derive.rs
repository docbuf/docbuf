use std::collections::{HashMap, HashSet};

use docbuf_core::vtable::*;

use proc_macro2::{token_stream, Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{DeriveInput, ItemStruct};

pub const DEFAULT_NAMESPACE: &str = "default";

#[derive(thiserror::Error, Debug)]
pub enum Error {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DocBufCryptoAlgorithm {
    Ed25519,
}

impl From<&str> for DocBufCryptoAlgorithm {
    fn from(algo: &str) -> Self {
        match algo {
            "ed25519" => DocBufCryptoAlgorithm::Ed25519,
            _ => unimplemented!("Unsupported crypto algorithm: {}", algo),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
}

impl From<&str> for HashAlgorithm {
    fn from(algo: &str) -> Self {
        match algo {
            "sha256" => HashAlgorithm::Sha256,
            _ => unimplemented!("Unsupported hash algorithm: {}", algo),
        }
    }
}

pub type HtmlTemplatePath = String;

pub type DbConfigPath = String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DocBufOpt {
    Namespace(String),
    Sign(bool),
    Crypto(DocBufCryptoAlgorithm),
    Hash(HashAlgorithm),
    Html(HtmlTemplatePath),
    UseUuid(bool),
    UseDb(bool),
    DbConfig(DbConfigPath),
}

#[derive(Debug, Clone)]
pub struct DocBufOpts(HashSet<DocBufOpt>);

impl DocBufOpts {
    pub fn namespace(&self) -> &str {
        self.0
            .iter()
            .find_map(|opt| match opt {
                DocBufOpt::Namespace(namespace) => Some(namespace.as_str()),
                _ => None,
            })
            .unwrap_or(DEFAULT_NAMESPACE)
    }

    /// Returns true if the `uuid` option is set to true,
    /// or if the `db_config` option is set.
    /// Otherwise, returns false.
    pub fn uuid(&self) -> bool {
        self.0.iter().any(|opt| match opt {
            DocBufOpt::DbConfig(_) => true,
            DocBufOpt::UseDb(v) => *v,
            DocBufOpt::UseUuid(v) => *v,
            _ => false,
        })
    }

    pub fn db_path(&self) -> Option<&str> {
        self.0.iter().find_map(|opt| match opt {
            DocBufOpt::DbConfig(path) => Some(path.as_str()),
            _ => None,
        })
    }

    pub fn db_mngr(&self) -> TokenStream {
        self.db_path()
            .map(|path| {
                quote! {
                    ::docbuf_db::DocBufDbManager::from_config(#path)?
                }
            })
            .unwrap_or(quote! {
                ::docbuf_db::DocBufDbManager::default()
            })
    }
}

impl<K, V> From<(K, V)> for DocBufOpt
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    fn from((key, value): (K, V)) -> Self {
        match (key.as_ref(), value.as_ref()) {
            ("namespace", namespace) => DocBufOpt::Namespace(namespace.to_string()),
            ("sign", v) => DocBufOpt::Sign(v == "true"),
            ("crypto", algo) => DocBufOpt::Crypto(DocBufCryptoAlgorithm::from(algo)),
            ("hash", algo) => DocBufOpt::Hash(HashAlgorithm::from(algo)),
            ("html", template) => DocBufOpt::Html(template.to_string()),
            ("uuid", v) => DocBufOpt::UseUuid(v == "true"),
            ("db", v) => DocBufOpt::UseDb(v == "true"),
            ("db_config", path) => {
                // unimplemented!("Db Config Path: {:?}", path);
                DocBufOpt::DbConfig(path.to_string())
            }
            _ => {
                unimplemented!("Unsupported DocBuf options key: {}", key.as_ref());
            }
        }
    }
}

impl From<&mut token_stream::IntoIter> for DocBufOpt {
    fn from(iter: &mut token_stream::IntoIter) -> Self {
        let key = iter.nth(0);
        let value = iter.nth(1);

        // drain the next token
        iter.next();

        match (key, value) {
            (Some(k), Some(v)) => {
                let key = k.to_string().replace("\"", "");
                let value = v.to_string().replace("\"", "");
                Self::from((key, value))
            }
            _ => {
                panic!("Failed to parse DocBuf options from token tree.")
            }
        }
    }
}

impl From<&TokenStream> for DocBufOpts {
    fn from(input: &TokenStream) -> Self {
        let mut iter = input.clone().into_iter();
        let mut num_attr = iter.clone().count() / 4;
        let mut opts = HashSet::new();

        while num_attr > 0 {
            opts.insert(DocBufOpt::from(&mut iter));

            num_attr -= 1;
        }

        Self(opts)
    }
}

// Parse the attributes of the derive macro
pub fn docbuf_attr(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    TokenStream::new()
}

pub fn docbuf_item(
    name: &TokenStream,
    lifetimes: &TokenStream,
    options: &DocBufOpts,
    item: &TokenStream,
) -> TokenStream {
    let fields = parse_item_fields(item, options);

    let derivatives = parse_item_derivatives(item);

    let output = quote! {
        #derivatives
        pub struct #name #lifetimes {
            #fields
        }
    };

    TokenStream::from(output)
}

pub fn docbuf_impl(
    name: &TokenStream,
    lifetimes: &TokenStream,
    options: &DocBufOpts,
    item: &TokenStream,
) -> TokenStream {
    let serialization_methods = docbuf_impl_serialization();
    let uuid_methods = docbuf_impl_uuid(options);
    let vtable = docbuf_impl_vtable(name, options, item);

    let output = quote! {
        impl #lifetimes ::docbuf_core::traits::DocBuf for #name #lifetimes {
            type Doc = Self;

            type DocId = [u8; 16];

            // Serialization Methods
            #serialization_methods

            // UUID Methods
            #uuid_methods

            // VTable
            #vtable
        }

    };

    TokenStream::from(output)
}

// Impl docbuf signing for the input struct
pub fn docbuf_impl_crypto(
    name: &TokenStream,
    lifetimes: &TokenStream,
    options: &DocBufOpts,
) -> TokenStream {
    let mut output = Vec::new();

    // Check if the sign option is present
    if options.0.contains(&DocBufOpt::Sign(true)) {
        output.push(quote! {
            impl #lifetimes ::docbuf_core::traits::DocBufCrypto for #name #lifetimes {}
        });
    }

    quote! {
        #(#output)*
    }
}

pub fn docbuf_impl_db(
    name: &TokenStream,
    lifetimes: &TokenStream,
    options: &DocBufOpts,
    item: &TokenStream,
) -> TokenStream {
    #[cfg(not(feature = "db"))]
    return TokenStream::new();

    // Check if the db_config option is present
    // let db_mngr = options.db_mngr();

    // Implement partition key, if it is present.
    let partition_key = docbuf_impl_partition(options, item);

    quote! {
        impl #lifetimes ::docbuf_db::DocBufDb for #name #lifetimes {
            type Db = ::docbuf_db::DocBufDbManager;

            /// The predicate type used for querying the database.
            type Predicate = ::docbuf_db::Predicates;

            /// Write a document into the database.
            /// This will return the document id.
            fn db_insert(&self, db: &Self::Db) -> Result<Self::DocId, ::docbuf_db::Error> {
                use ::docbuf_db::DocBufDbMngr;

                // Setup the vtable for the document.
                let vtable = Self::vtable()?;
                db.write_vtable(&vtable)?;

                db.insert(self, self.partition_key()?)
            }

            /// Return all documents in the database.
            fn db_all(
                db: &Self::Db,
                partition_key: Option<impl Into<::docbuf_db::PartitionKey>>
            ) -> Result<impl Iterator<Item = Self::DocId>, ::docbuf_db::Error>
            {
                use ::docbuf_db::DocBufDbMngr;

                db.all::<Self::Doc>(partition_key.map(|p| p.into())).map(|ids| ids.into_iter())
            }

            /// Read documents in the database given a predicate.
            fn db_find(
                db: &Self::Db,
                predicate: impl Into<Self::Predicate>,
                partition_key: Option<impl Into<::docbuf_db::PartitionKey>>
            ) -> Result<impl Iterator<Item = Self::Doc>, ::docbuf_db::Error>
            {
                use ::docbuf_db::DocBufDbMngr;

                db.find::<Self::Doc>(predicate.into(), partition_key.map(|p| p.into()))
            }

            /// Get a document from the database.
            fn db_get(
                db: &Self::Db,
                id: Self::DocId,
                partition_key: Option<impl Into<::docbuf_db::PartitionKey>>
            ) -> Result<Option<Self::Doc>, ::docbuf_db::Error> {
                use ::docbuf_db::DocBufDbMngr;

                let doc = db.get::<Self::Doc>(id, partition_key.map(|p| p.into()))?;

                Ok(doc)
            }

            /// Update a document in the database.
            fn db_update(&self, db: &Self::Db) -> Result<(), ::docbuf_db::Error> {
                use ::docbuf_db::DocBufDbMngr;

                db.update::<Self::Doc>(self, self.partition_key()?)
            }

            /// Delete a document from the database.
            fn db_delete(self, db: &Self::Db) -> Result<Self::Doc, ::docbuf_db::Error> {
                use ::docbuf_db::DocBufDbMngr;

                let partition_key = self.partition_key()?;

                db.delete::<Self::Doc>(self, partition_key)
            }

            /// Delete a document partition from the database.
            fn db_delete_partition(db: &Self::Db, partition_key: impl Into<::docbuf_db::PartitionKey>) -> Result<(), ::docbuf_db::Error> {
                unimplemented!("DocBufDb method not implemented")
            }

            /// Return the number of documents in the database.
            fn db_count(db: &Self::Db, partition_key: Option<impl Into<::docbuf_db::PartitionKey>>) -> Result<usize, ::docbuf_db::Error> {
                use ::docbuf_db::DocBufDbMngr;

                db.count::<Self::Doc>(partition_key.map(|p| p.into()))
            }

            /// Return the number of documents in the database given a predicate.
            fn db_count_where(
                db: &Self::Db,
                predicate: Self::Predicate,
            ) -> Result<usize, ::docbuf_db::Error> {
                unimplemented!("DocBufDb method not implemented")
            }

            /// Return the number of documents in the database given a partition key.
            fn db_count_partition(db: &Self::Db, partition_key: impl Into<::docbuf_db::PartitionKey>) -> Result<usize, ::docbuf_db::Error> {
                unimplemented!("DocBufDb method not implemented")
            }

            #partition_key

        }
    }
}

pub fn docbuf_impl_partition(options: &DocBufOpts, item: &TokenStream) -> TokenStream {
    let ast: ItemStruct = syn::parse(item.to_owned().into()).expect("Failed to parse item");

    ast.fields
        .into_iter()
        .find_map(|field| {
            field.attrs.iter().find_map(|attr| {
                let attributes = parse_docbuf_field_attrs(attr.to_token_stream())
                    .expect("Failed to parse field attributes");

                attributes
                    .iter()
                    .find_map(|(_, (key, value))| match key.to_string().as_str() {
                        "partition_key" => {
                            if value.to_string().as_str() != "true" {
                                return None;
                            }

                            let field_name = field.ident.as_ref().expect("Field name not found for partition key");
                            Some(quote! {
                                /// Return the partition key for the document.
                                fn partition_key(&self) -> Result<::docbuf_db::PartitionKey, ::docbuf_db::Error> {
                                    Ok(::docbuf_db::PartitionKey::from(self.#field_name.clone()))
                                }
                            })
                        }
                        _ => None,
                    })
            })
        })
        .unwrap_or(quote! {})
}

pub fn docbuf_impl_vtable(
    name: &TokenStream,
    options: &DocBufOpts,
    item: &TokenStream,
) -> TokenStream {
    let namespace = options.namespace();

    let ast: ItemStruct = syn::parse(item.to_owned().into()).expect("Failed to parse item");

    let fields = ast.fields.into_iter().collect::<Vec<_>>();

    let fields = fields.iter().map(|field| {
        let name = field.ident.as_ref().map(|f| f.to_owned()).unwrap_or(Ident::new("__inner__", Span::call_site()));
        let ty = field.ty.to_token_stream();
        let rules = parse_field_rules(&field).expect("Failed to parse field rules");

        match VTableFieldType::is_struct(ty.to_string().as_ref()) {
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
                                let name = #struct_name_var.name.clone();
                                if name == stringify!(#ty) {
                                    let field_type = ::docbuf_core::vtable::VTableFieldType::Struct(name);

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

    // Add the `_uuid` field to the vtable if the option is enabled
    let uuid = match options.uuid() {
        true => quote! {
            let field_type = ::docbuf_core::vtable::VTableFieldType::Uuid;
            let field_rules = ::docbuf_core::vtable::VTableFieldRules::new();
            vtable_struct.add_field(field_type, "_uuid", field_rules);
        },
        false => quote! {},
    };

    let vtable = quote! {
        fn vtable() -> Result<&'static ::docbuf_core::vtable::VTable, ::docbuf_core::error::Error> {
            static VTABLE: ::std::sync::OnceLock<::docbuf_core::vtable::VTable> = ::std::sync::OnceLock::new();

            let vtable = VTABLE.get_or_init(||  {
                let mut vtable = ::docbuf_core::vtable::VTable::new(String::from(#namespace), String::from(stringify!(#name)));

                let mut vtable_struct = ::docbuf_core::vtable::VTableStruct::new(stringify!(#name), None);

                // Add the _uuid field to the vtable
                #uuid

                // Add the fields to the vtable
                #(#fields)*

                // Sorting is required to ensure the structs are added in a consistent order
                vtable.items.inner_mut().sort_by(|a, b| match (a, b) {
                    (::docbuf_core::vtable::VTableItem::Struct(a), ::docbuf_core::vtable::VTableItem::Struct(b)) => a.name
                        .cmp(&b.name)
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

// Impl docbuf return uuid
pub fn docbuf_impl_uuid(options: &DocBufOpts) -> TokenStream {
    if options.uuid() {
        let output = quote! {
            fn uuid(&self) -> Result<Self::DocId, ::docbuf_core::error::Error> {
                Ok(self._uuid)
            }
        };

        TokenStream::from(output)
    } else {
        TokenStream::new()
    }
}

// Impl docbuf serialization and deserialization for the input struct
pub fn docbuf_impl_serialization() -> TokenStream {
    let output = quote! {
        // Serialize the struct to a byte buffer
        fn to_docbuf<'a>(&self, buffer: &'a mut Vec<u8>) -> Result<::docbuf_core::vtable::VTableFieldOffsets, ::docbuf_core::error::Error> {
            let offsets = ::docbuf_core::serde::ser::to_docbuf(self, buffer)?;

            Ok(offsets)
        }

        // Deserialize the byte buffer to a struct
        fn from_docbuf<'a>(buf: &'a mut Vec<u8>) -> Result<Self, ::docbuf_core::error::Error> {
            Ok(::docbuf_core::serde::de::from_docbuf(buf)?)
        }
    };

    TokenStream::from(output)
}

pub fn derive_docbuf(attr: TokenStream, item: TokenStream) -> TokenStream {
    let name = parse_item_name(&item);
    let lifetimes = parse_item_lifetimes(&item);

    let options = DocBufOpts::from(&attr);

    let item_docbuf = docbuf_item(&name, &lifetimes, &options, &item);
    let docbuf_methods = docbuf_impl(&name, &lifetimes, &options, &item);
    let crypto_methods = docbuf_impl_crypto(&name, &lifetimes, &options);
    let db_methods = docbuf_impl_db(&name, &lifetimes, &options, &item);

    let output = quote! {
        #item_docbuf
        #docbuf_methods
        #crypto_methods
        #db_methods
    };

    TokenStream::from(output)
}

// Parse the item name from the input token stream
pub fn parse_item_name(input: &TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.to_owned().into()).expect("Failed to parse input");
    ast.ident.to_token_stream()
}

// Retain the item derive macros from the input token stream,
// stripping the docbuf derive macro.
pub fn parse_item_derivatives(input: &TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.to_owned().into()).expect("Failed to parse input");
    let derivatives = ast.attrs.iter().filter_map(|attr| {
        if !attr.path().is_ident("docbuf") {
            Some(attr.to_token_stream())
        } else {
            None
        }
    });

    quote! {
        #(#derivatives)*
    }
}

// Parse the item fields from the input stream
pub fn parse_item_fields(item: &TokenStream, options: &DocBufOpts) -> TokenStream {
    let ast: ItemStruct = syn::parse(item.to_owned().into()).expect("Failed to parse item fields.");
    let fields = ast.fields.iter().map(|field| {
        let name = field
            .ident
            .as_ref()
            .map(|f| f.to_owned())
            .unwrap_or(Ident::new("__inner__", Span::call_site()));
        let ty = field.ty.to_token_stream();
        let vis = &field.vis;

        // parse the field attributes, e.g. `#[serde()]`
        let attr = field
            .attrs
            .iter()
            .filter_map(|attr| {
                // Check if attr is `docbuf` attribute
                if !attr.path().is_ident("docbuf") {
                    Some(attr.to_token_stream())
                } else {
                    Some(quote! {})
                }
            })
            .collect::<Vec<TokenStream>>();

        // TODO: Parse comments from the field attributes

        quote! {
            #(#attr)*
            #vis #name: #ty
        }
    });

    // Check for added fields
    let id_field = match options.uuid() {
        true => quote! {
            /// DocBuf Universal Doc ID
            // #[serde(with = "::docbuf_core::deps::uuid::serde::compact")]
            // pub _uuid: ::docbuf_core::deps::uuid::Uuid,
            #[serde(with = "::docbuf_core::serde::serde_bytes")]
            pub _uuid: [u8; 16],
        },
        false => quote! {},
    };

    quote! {
        #id_field

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
                .filter_map(|(_, (key, value))| match key.to_string().as_str() {
                    "sign" | "ignore" => Some(quote! {
                        field_rules.#key = #value;
                    }),
                    "min_value" | "max_value" => Some(quote! {
                        field_rules.#key = Some((#value as #field_type).into());
                    }),
                    "min_length" | "max_length" | "length" => Some(quote! {
                        field_rules.#key = Some(#value);
                    }),
                    #[cfg(feature = "regex")]
                    "regex" => Some(quote! {
                        field_rules.#key = Some(
                            #value.to_string()
                            // ::docbuf_core::validate::regex::Regex::new(#value)
                            //     .expect(&format!("Invalid regex: {:?}", #value))
                        );
                    }),
                    _ => None,
                })
                .collect::<Vec<TokenStream>>();

            Ok(quote!(
                #(#rules)*
            ))
        })
        .collect::<Result<Vec<TokenStream>, Error>>()?;

    let rules = quote!(
        let mut field_rules = ::docbuf_core::vtable::VTableFieldRules::new();

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
