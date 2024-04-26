use std::ops::Deref;

use quiche::h3::NameValue;

use crate::Error;

/// Default Rpc Header is an alias of quiche::h3::Header, which is simply a tuple of bytes, i.e. (Vec<u8>, Vec<u8>).
pub type RpcHeader = quiche::h3::Header;

#[derive(Debug, Clone)]
pub struct RpcHeaders<Header> {
    inner: Vec<Header>,
}

impl Deref for RpcHeaders<RpcHeader> {
    type Target = Vec<RpcHeader>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Vec<RpcHeader>> for RpcHeaders<RpcHeader> {
    fn from(inner: Vec<RpcHeader>) -> Self {
        RpcHeaders { inner }
    }
}

impl RpcHeaders<RpcHeader> {
    pub fn service(&self) -> Result<&str, Error> {
        self.inner
            .iter()
            .find(|header| header.name() == b":path")
            .map(|header| std::str::from_utf8(header.value()))
            .transpose()
            .map_err(|_| Error::InvalidHeader)?
            .ok_or(Error::MissingHeader(":path".into()))
    }

    pub fn method(&self) -> Result<&str, Error> {
        self.inner
            .iter()
            .find(|header| header.name() == b":method")
            .map(|header| std::str::from_utf8(header.value()))
            .transpose()
            .map_err(|_| Error::InvalidHeader)?
            .ok_or(Error::MissingHeader(":method".into()))
    }
}
