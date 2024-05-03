use std::ops::Deref;

use quiche::h3::NameValue;

use crate::Error;

/// Default Rpc Header is an alias of quiche::h3::Header, which is simply a tuple of bytes, i.e. `(Vec<u8>, Vec<u8>)`.
pub type RpcHeader = quiche::h3::Header;

#[derive(Debug, Clone)]
pub struct RpcHeaders {
    pub(crate) inner: Vec<RpcHeader>,
}

impl Default for RpcHeaders {
    fn default() -> Self {
        RpcHeaders {
            inner: vec![
                RpcHeader::new(b":method", b"POST"),
                RpcHeader::new(b":scheme", b"http"),
                RpcHeader::new(b"content-type", b"application/docbuf+rpc"),
                RpcHeader::new(b"user-agent", b"docbuf-rpc-client/0.1.0"),
            ],
        }
    }
}

impl RpcHeaders {
    pub fn with_path(mut self, path: &str) -> Self {
        self.inner.push(RpcHeader::new(b":path", path.as_bytes()));

        self
    }

    /// Add a content length header to the headers.
    pub fn with_content_length(mut self, content_length: usize) -> Self {
        self.inner.push(RpcHeader::new(
            b"content-length",
            content_length.to_string().as_bytes(),
        ));
        self
    }
}

impl Deref for RpcHeaders {
    type Target = Vec<RpcHeader>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Vec<RpcHeader>> for RpcHeaders {
    fn from(inner: Vec<RpcHeader>) -> Self {
        RpcHeaders { inner }
    }
}

impl RpcHeaders {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the UTF-8 string encoded content-length of the body,
    /// if the `content-length: x` value has been set.
    pub fn content_length(&self) -> Option<usize> {
        self.inner
            .iter()
            .find(|header| header.name() == b"content-length")
            .and_then(|header| {
                std::str::from_utf8(header.value())
                    .map_or(None, |value| value.parse::<usize>().ok())
            })
    }

    pub fn path(&self) -> Result<&str, Error> {
        self.inner
            .iter()
            .find(|header| header.name() == b":path")
            .map(|header| std::str::from_utf8(header.value()))
            .transpose()
            .map_err(|_| Error::InvalidHeader)?
            .ok_or(Error::MissingHeader(":path".into()))
    }

    /// Returns the service from the path header.
    pub fn service(&self) -> Result<&str, Error> {
        // service is the second to last part of the path,
        // e.g. /v1/service/method
        self.path()?
            .split('/')
            .nth_back(1)
            .ok_or(Error::InvalidHeader)
    }

    /// Returns the method from the path header.
    pub fn method(&self) -> Result<&str, Error> {
        self.path()?
            .split('/')
            .nth_back(0)
            .ok_or(Error::InvalidHeader)
    }
}
