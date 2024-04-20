use std::net::SocketAddr;

pub struct RpcAuthToken<Id> {
    pub(crate) id: Id,
    pub(crate) addr: SocketAddr,
}

impl RpcAuthToken<quiche::ConnectionId<'static>> {
    pub fn new(id: quiche::ConnectionId<'static>, addr: SocketAddr) -> Self {
        Self { id, addr }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.id.to_vec()
    }

    pub fn from_vec(id: Vec<u8>, addr: SocketAddr) -> Self {
        Self {
            id: quiche::ConnectionId::from_vec(id),
            addr,
        }
    }
}
