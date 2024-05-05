use crate::{Error, RpcHeader, RpcRequest, RpcResponse};

use std::{collections::HashMap, sync::Arc};

// pub type Future<Res> =
//     std::pin::Pin<Box<dyn std::future::Future<Output = Result<Res, Error>> + Send>>;

// pub type Iter<Res> = std::pin::Pin<Box<dyn std::iter::Iterator<Item = Result<Res, Error>> + Send>>;

pub type RpcResult = Result<RpcResponse, Error>;

pub type RpcMethodHandler<Ctx> = Arc<dyn Fn(Ctx, RpcRequest) -> RpcResult + Send + Sync + 'static>;

/// the name of a service.
pub type RpcServiceName = String;

/// The name of the method in a service.
pub type RpcMethodName = String;

// An RpcService is a collection of RPC Methods.
pub struct RpcService<Ctx> {
    name: RpcServiceName,
    inner: HashMap<RpcMethodName, RpcMethodHandler<Ctx>>,
}

impl<Ctx> RpcService<Ctx>
where
    Ctx: Send + Sync + 'static,
{
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            inner: HashMap::new(),
        }
    }

    pub fn name(&self) -> &RpcServiceName {
        &self.name
    }

    pub fn add_method(
        mut self,
        method_name: &str,
        method: impl Fn(Ctx, RpcRequest) -> RpcResult + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        self.inner.insert(method_name.to_owned(), Arc::new(method));
        Ok(self)
    }
}

impl<Ctx> std::fmt::Debug for RpcService<Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcService")
            .field("name", &self.name)
            .finish()
    }
}

/// A map of service names to a map of method names to service methods.
pub struct RpcServices<Ctx>
where
    Ctx: Clone + Send + Sync + 'static,
{
    pub(crate) ctx: Ctx,
    inner: HashMap<RpcServiceName, RpcService<Ctx>>,
}

impl<Ctx> RpcServices<Ctx>
where
    Ctx: Clone + Send + Sync + 'static,
{
    pub fn new(ctx: Ctx) -> Self {
        Self {
            ctx,
            inner: HashMap::new(),
        }
    }

    pub fn add_service_method(
        &mut self,
        service_name: &str,
        method_name: &str,
        method: RpcMethodHandler<Ctx>,
    ) -> Result<(), Error> {
        let service = self
            .inner
            .entry(service_name.to_owned())
            .or_insert_with(|| RpcService::new(service_name));
        service.inner.insert(method_name.to_owned(), method);
        Ok(())
    }

    pub fn get_service(&self, service_name: &str) -> Option<&RpcService<Ctx>> {
        self.inner.get(service_name)
    }

    pub fn get_method(
        &self,
        service_name: &str,
        method_name: &str,
    ) -> Option<&RpcMethodHandler<Ctx>> {
        self.inner
            .get(service_name)
            .and_then(|service| service.inner.get(method_name))
    }

    pub fn add_service(mut self, service: RpcService<Ctx>) -> Result<Self, Error> {
        self.inner.insert(service.name.to_owned(), service);

        Ok(self)
    }

    // Merge the RpcServices from another RpcServices into this RpcServices.
    pub fn merge_services(mut self, other: RpcServices<Ctx>) -> Result<Self, Error> {
        for (service_name, methods) in other.inner {
            for (method_name, method) in methods.inner {
                self.add_service_method(&service_name, &method_name, method)?;
            }
        }

        Ok(self)
    }
}
