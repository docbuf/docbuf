use crate::{Error, RpcHeader, RpcRequest, RpcResponse};

use std::{collections::HashMap, sync::Arc};

pub type Future<Res> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<Res, Error>> + Send>>;

pub type Iter<Res> = std::pin::Pin<Box<dyn std::iter::Iterator<Item = Result<Res, Error>> + Send>>;

pub enum RpcResult<Res> {
    Future(Future<Res>),
    Iter(Iter<Res>),
}

pub type RpcMethodHandler<Ctx> = Arc<
    dyn Fn(Ctx, RpcRequest<RpcHeader>) -> RpcResult<RpcResponse<RpcHeader>> + Send + Sync + 'static,
>;

/// the name of a service.
pub type RpcServiceName = String;

/// The name of the method in a service.
pub type RpcMethodName = String;

// An RpcService is a collection of RPC Methods.
pub struct RpcService<Ctx>(HashMap<RpcMethodName, RpcMethodHandler<Ctx>>);

impl<Ctx> RpcService<Ctx>
where
    Ctx: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_method(
        mut self,
        method_name: &str,
        method: impl Fn(Ctx, RpcRequest<RpcHeader>) -> RpcResult<RpcResponse<RpcHeader>>
            + Send
            + Sync
            + 'static,
    ) -> Result<Self, Error> {
        self.0.insert(method_name.to_owned(), Arc::new(method));
        Ok(self)
    }
}

/// A map of service names to a map of method names to service methods.
pub struct RpcServices<Ctx>(HashMap<RpcServiceName, RpcService<Ctx>>)
where
    Ctx: Clone + Send + Sync + 'static;

impl<Ctx> RpcServices<Ctx>
where
    Ctx: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_service_method(
        &mut self,
        service_name: &str,
        method_name: &str,
        method: RpcMethodHandler<Ctx>,
    ) -> Result<(), Error> {
        let service = self
            .0
            .entry(service_name.to_owned())
            .or_insert_with(|| RpcService::new());
        service.0.insert(method_name.to_owned(), method);
        Ok(())
    }

    pub fn get_service(&self, service_name: &str) -> Option<&RpcService<Ctx>> {
        self.0.get(service_name)
    }

    pub fn get_method(
        &self,
        service_name: &str,
        method_name: &str,
    ) -> Option<&RpcMethodHandler<Ctx>> {
        self.0
            .get(service_name)
            .and_then(|service| service.0.get(method_name))
    }

    pub fn add_service(
        mut self,
        service_name: &str,
        service: RpcService<Ctx>,
    ) -> Result<Self, Error> {
        self.0.insert(service_name.to_owned(), service);

        Ok(self)
    }

    // Merge the RpcServices from another RpcServices into this RpcServices.
    pub fn merge_services(mut self, other: RpcServices<Ctx>) -> Result<Self, Error> {
        for (service_name, methods) in other.0 {
            for (method_name, method) in methods.0 {
                self.add_service_method(&service_name, &method_name, method)?;
            }
        }

        Ok(self)
    }
}
