use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use proc_macro2::{token_stream, Ident, Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::{ItemImpl, ItemTrait, Signature};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum RpcOption {
    /// Service name
    Service(String),
    GenClient(bool),
    None,
}

impl<K, V> From<(K, V)> for RpcOption
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    fn from((key, value): (K, V)) -> Self {
        match (key.as_ref(), value.as_ref()) {
            ("service", service) => RpcOption::Service(service.to_string()),
            ("gen_client", val) => RpcOption::GenClient(val == "true"),
            _ => {
                unimplemented!("Unsupported DocBuf RPC options key: {}", key.as_ref());
            }
        }
    }
}

pub struct RpcOptions(HashSet<RpcOption>);

impl RpcOptions {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn insert(&mut self, option: RpcOption) {
        self.0.insert(option);
    }

    pub fn gen_client(&self) -> bool {
        self.iter()
            .filter_map(|opt| match opt {
                RpcOption::GenClient(gen) => Some(*gen),
                _ => None,
            })
            .next()
            // default behavior is to generate the client.
            .unwrap_or(true)
    }

    // Get the service name from the options
    pub fn service(&self) -> Option<&str> {
        self.iter()
            .filter_map(|opt| match opt {
                RpcOption::Service(service) => Some(service.as_str()),
                _ => None,
            })
            .next()
    }

    /// Set the service name in the options.
    /// This will overwrite any existing service name.
    pub fn set_service(&mut self, svc: String) {
        self.0.retain(|opt| match opt {
            RpcOption::Service(_) => false,
            _ => true,
        });

        self.insert(RpcOption::Service(svc));
    }
}

impl Deref for RpcOptions {
    type Target = HashSet<RpcOption>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RpcOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&mut token_stream::IntoIter> for RpcOptions {
    fn from(iter: &mut token_stream::IntoIter) -> Self {
        let mut options = RpcOptions::new();

        loop {
            let key = iter.nth(0);
            let value = iter.nth(1);

            if let (Some(key), Some(value)) = (key, value) {
                let key = key.to_string().replace("\"", "");
                let value = value.to_string().replace("\"", "");

                println!("Found RPC Option: {key} = {value}");

                options.insert(RpcOption::from((key, value)));
            }

            if iter.next().is_none() {
                break;
            }
        }

        options
    }
}

impl From<&TokenStream> for RpcOptions {
    fn from(input: &TokenStream) -> Self {
        let mut iter = input.clone().into_iter();
        RpcOptions::from(&mut iter)
    }
}

fn check_method_sig(signature: &syn::Signature) -> Result<(), ()> {
    if signature.inputs.len() != 2 {
        panic!(
            "`{}` has invalid function definition. The method should only accept `ctx` and `mut doc` arguments, e.g., \n
`fn {}<Ctx, Doc>(ctx: &Ctx, doc: mut Doc) -> Result<(), docbuf_rpc::Error> \n
    where
        Ctx: Send + Sync + 'static,
        Doc: Docbuf, \n
{{ ... }}`",
            signature.ident.to_token_stream(),
            signature.ident.to_token_stream(),
        )
    } else {
        Ok(())
    }
}

fn parse_ctx_type(signature: &syn::Signature) -> Result<syn::Type, ()> {
    match signature.inputs.first() {
        Some(syn::FnArg::Typed(typed)) => Ok(*typed.ty.to_owned()),
        _ => {
            panic!(
                "Expected first instance to be `Ctx` context type. Found: {}",
                signature.ident.to_token_stream(),
            )
        }
    }
}

fn parse_doc_type(signature: &syn::Signature) -> Result<syn::Type, ()> {
    match signature.inputs.last() {
        Some(syn::FnArg::Typed(typed)) => Ok(*typed.ty.to_owned()),
        _ => Err(emit_error!(
            signature.ident,
            "Expected first instance to be `Ctx` context type."
        )),
    }
}

fn rpc_function(
    signature: &syn::Signature,
    ctx: &syn::Type,
    doc: &syn::Type,
) -> Result<TokenStream, ()> {
    let method = signature.ident.to_token_stream();
    let rpc_method = format!("rpc_{}", method.to_string()).to_lowercase();
    let rpc_method_var = Ident::new(&rpc_method, Span::call_site());

    let ctx_type = ctx.to_token_stream();
    let doc_type = doc.to_token_stream();

    Ok(quote! {
        pub fn #rpc_method_var(
            ctx: #ctx_type,
            mut req: docbuf_rpc::RpcRequest
        ) -> docbuf_rpc::RpcResult
        {
            let document = Self::#method(ctx, req.as_docbuf::<#doc_type>()?)?;

            // Instantiate the response content body buffer to the length of the content length.
            let mut buffer = #doc_type::vtable()?.alloc_buf();

            document.to_docbuf(&mut buffer)?;

            let headers = docbuf_rpc::RpcHeaders::default()
                .with_path(req.headers.path()?)
                .with_content_length(buffer.len());

            Ok(docbuf_rpc::RpcResponse::new(req.stream_id, headers, buffer))
        }
    })
}

fn parse_rpc_service(
    ctx_type: &syn::Type,
    service_name: &str,
    methods: &[Signature],
) -> Result<TokenStream, ()> {
    let service_name = service_name.to_lowercase();

    let rpc_methods = methods
        .iter()
        .map(|sig| {
            let method = sig.ident.to_string();
            let rpc_method = format!("rpc_{}", method.to_string()).to_lowercase();
            let rpc_method_var = Ident::new(&rpc_method, Span::call_site());

            Ok(quote! {
                .add_method(#method, Self::#rpc_method_var)?
            })
        })
        .collect::<Result<Vec<TokenStream>, ()>>()?;

    Ok(quote! {
        pub fn rpc_service() -> Result<docbuf_rpc::RpcService<#ctx_type>, Error>
            {
                Ok(docbuf_rpc::RpcService::new(#service_name)
                    #(#rpc_methods)*)
            }
    })
}

fn parse_method_signatures(input: &ItemImpl) -> Vec<Signature> {
    input
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(f) => Some(f),
            _ => None,
        })
        .map(|f| f.sig.clone())
        .collect::<Vec<Signature>>()
}

pub fn gen_rpc(attr: TokenStream, input: TokenStream) -> Result<TokenStream, ()> {
    let mut options = RpcOptions::from(&attr);

    // Generate the RPC Service if the item is a impl block.
    if let Ok(implementation) = syn::parse2::<ItemImpl>(input.clone()) {
        let (service, client) = gen_rpc_service_and_client(implementation, &mut options)?;

        return Ok(quote! {
            #input
            #service
            #client
        });
    }

    // Generate the RPC Client if the item is a trait block.
    if let Ok(interface) = syn::parse2::<ItemTrait>(input.clone()) {
        return gen_rpc_client(interface, &mut options);
    }

    panic!("Expected either an `impl` block or a `trait` block.")
}

pub fn gen_rpc_client(interface: ItemTrait, options: &mut RpcOptions) -> Result<TokenStream, ()> {
    // Trait Name
    let trait_name = interface.ident;

    let methods = interface
        .items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Fn(f) => Some(f),
            _ => None,
        })
        .map(|f| {
            let sig = &f.sig;

            let doc_type = parse_doc_type(sig)?;
            let service = options.service().unwrap_or_else(|| {
                panic!(
                    r#"RPC `service` name must be provided. E.g.,
#[docbuf_rpc {{
    service = "service_name";
}}]"#
                );
            });
            let method = sig.ident.to_string();
            let path = format!("/{service}/{method}");

            Ok(quote! {
                #sig {
                    let mut buffer = #doc_type::vtable()?.alloc_buf();
                    doc.to_docbuf(&mut buffer)?;

                    let headers = docbuf_rpc::RpcHeaders::default()
                        .with_path(#path)
                        .with_content_length(buffer.len());

                    let request = docbuf_rpc::RpcRequest::default()
                        .add_headers(headers)
                        .add_body(buffer);

                    let mut response = client.send(request)?;

                    let doc = #doc_type::from_docbuf(&mut response.body)?;

                    Ok(doc)
                }
            })
        })
        .collect::<Result<Vec<TokenStream>, ()>>()?;

    Ok(quote! {
        pub trait #trait_name {
            #(#methods)*
        }
    })
}

fn client_function(signature: &syn::Signature) -> Result<TokenStream, ()> {
    let method = signature.ident.to_token_stream();

    let doc = parse_doc_type(signature)?;

    Ok(quote! {
        fn #method(
            client: &docbuf_rpc::RpcClient,
            doc: #doc
        ) -> Result<#doc, docbuf_rpc::Error>;
    })
}

fn gen_rpc_client_interface(
    service: &str,
    signatures: &[Signature],
    options: &mut RpcOptions,
) -> Result<TokenStream, ()> {
    let client_name = format!("{}Client", service.to_string());
    let client_name = Ident::new(&client_name, Span::call_site());

    let methods = signatures
        .iter()
        .map(|sig| client_function(sig))
        .collect::<Result<Vec<TokenStream>, ()>>()?;

    let trait_def = quote! {
        pub trait #client_name {
            #(#methods)*
        }
    };

    // Set the service name in the options to the service passed to the method.
    options.set_service(service.to_lowercase());

    match syn::parse2::<ItemTrait>(trait_def) {
        Ok(interface) => gen_rpc_client(interface, options),
        Err(e) => panic!("Expected a trait block for {client_name}: {e:?}."),
    }
}

pub fn gen_rpc_service_and_client(
    implementation: ItemImpl,
    options: &mut RpcOptions,
) -> Result<(TokenStream, TokenStream), ()> {
    let signatures = parse_method_signatures(&implementation);

    let item_ident = implementation.self_ty.to_token_stream();

    let rpc_methods = signatures
        .iter()
        .map(|sig| {
            check_method_sig(&sig)?;
            let ctx_type = parse_ctx_type(&sig)?;
            let doc_type = parse_doc_type(&sig)?;
            let rpc_method = rpc_function(&sig, &ctx_type, &doc_type)?;

            Ok(rpc_method)
        })
        .collect::<Result<Vec<TokenStream>, ()>>()?;

    let service_name_string = format!("{}", item_ident.to_string());
    let service_name = Ident::new(&service_name_string, Span::call_site());

    let rpc_service = parse_rpc_service(
        &parse_ctx_type(&signatures[0])?,
        &service_name.to_string(),
        &signatures,
    )?;

    let service = quote! {
        impl #item_ident {
            #rpc_service

            #(#rpc_methods)*
        }
    };

    let client = if options.gen_client() {
        gen_rpc_client_interface(&service_name_string, &signatures, options)?
    } else {
        TokenStream::new()
    };

    Ok((service, client))
}
