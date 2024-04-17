use crate::error::Error;
// use crate::http3::Http3Config;
use crate::quic::{QuicConfig, TlsOptions, TransportErrorCode, MAX_QUIC_DATAGRAM_SIZE};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Duration;

use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};
use quiche::PROTOCOL_VERSION;
use tracing::{debug, error, info, warn};

/// Default token prefix.
pub const DEFAULT_TOKEN_PREFIX: &[u8; 10] = b"docbuf-rpc";

/// Maximum UDP payload size.
pub const MAX_UDP_PAYLOAD_SIZE: usize = u16::MAX as usize;

/// Default socket token.
const DEFAULT_SOCKET: Token = Token(0);

/// Default events capacity.
const DEFAULT_EVENTS_CAPACITY: usize = 1024;

/// Default polling period.
const DEFAULT_POLL_WINDOW: Duration = Duration::from_millis(50);

pub type StreamId = u64;

pub struct RpcAuthToken<Id> {
    id: Id,
    addr: SocketAddr,
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

#[derive(Debug, Clone)]
pub struct RpcRequest<Header: Clone + std::fmt::Debug> {
    stream_id: StreamId,
    headers: Vec<Header>,
    body: Option<Vec<u8>>,
}

impl<Header> RpcRequest<Header>
where
    Header: Clone + std::fmt::Debug,
{
    pub fn with_body(stream_id: StreamId, headers: Vec<Header>, body: Vec<u8>) -> Self {
        Self {
            stream_id,
            headers,
            body: Some(body),
        }
    }

    pub fn without_body(stream_id: StreamId, headers: Vec<Header>) -> Self {
        Self {
            stream_id,
            headers,
            body: None,
        }
    }
}

pub struct RpcResponse<Header> {
    stream_id: StreamId,
    headers: Vec<Header>,
    body: Vec<u8>,
}

impl<Header> RpcResponse<Header> {
    pub fn new(stream_id: StreamId, headers: Vec<Header>, body: Vec<u8>) -> Self {
        Self {
            stream_id,
            headers,
            body,
        }
    }

    // If the response is partially written, return a partial response, setting the bytes_written.
    pub fn consumed_partial(self, bytes_written: usize) -> RpcPartialResponse<Header> {
        RpcPartialResponse {
            stream_id: self.stream_id,
            headers: None,
            body: self.body,
            written: bytes_written,
        }
    }
}

pub struct RpcPartialResponse<Header> {
    stream_id: StreamId,
    headers: Option<Vec<Header>>,
    body: Vec<u8>,
    written: usize,
}

impl<Header> Into<RpcPartialResponse<Header>> for RpcResponse<Header> {
    fn into(self) -> RpcPartialResponse<Header> {
        RpcPartialResponse {
            stream_id: self.stream_id,
            headers: Some(self.headers),
            body: self.body,
            written: 0,
        }
    }
}

impl<Header> RpcPartialResponse<Header> {
    pub fn new(stream_id: StreamId) -> Self {
        Self {
            stream_id,
            headers: None,
            body: Vec::new(),
            written: 0,
        }
    }
}

pub struct RpcPartialResponses<T>(HashMap<StreamId, RpcPartialResponse<T>>);

impl<T> RpcPartialResponses<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, partial: RpcPartialResponse<T>) {
        self.0.insert(partial.stream_id, partial);
    }

    pub fn remove(&mut self, stream_id: StreamId) -> Option<RpcPartialResponse<T>> {
        self.0.remove(&stream_id)
    }
}

pub struct RpcConnection<Conn, Http3Conn, Header> {
    inner: Conn,
    http3: Option<Http3Conn>,
    partial_responses: RpcPartialResponses<Header>,
}

impl From<quiche::Connection>
    for RpcConnection<quiche::Connection, quiche::h3::Connection, quiche::h3::Header>
{
    fn from(inner: quiche::Connection) -> Self {
        Self::new(inner)
    }
}

impl RpcConnection<quiche::Connection, quiche::h3::Connection, quiche::h3::Header> {
    pub fn new(inner: quiche::Connection) -> Self {
        Self {
            inner,
            http3: None,
            partial_responses: RpcPartialResponses::new(),
        }
    }

    /// Check if the HTTP/3 connection is established.
    pub fn is_http3_established(&self) -> bool {
        self.http3.is_some() && (self.inner.is_in_early_data() || self.inner.is_established())
    }

    /// Set the HTTP/3 Connection.
    pub fn set_http3(&mut self, http3: quiche::h3::Connection) {
        self.http3 = Some(http3);
    }

    /// Get the HTTP/3 Connection.
    pub fn http3(&mut self) -> Option<&mut quiche::h3::Connection> {
        self.http3.as_mut()
    }

    /// Send an HTTP/3 Response.
    pub fn handle_response(
        &mut self,
        response: RpcResponse<quiche::h3::Header>,
    ) -> Result<(), Error> {
        let http3_cx = self.http3.as_mut().ok_or(Error::Http3ConnectionNotFound)?;

        // Send response headers
        match http3_cx.send_response(
            &mut self.inner,
            response.stream_id,
            &response.headers,
            false,
        ) {
            Ok(()) => {}

            Err(quiche::h3::Error::StreamBlocked) => {
                self.partial_responses.insert(response.into());
                return Ok(());
            }

            Err(e) => {
                error!(
                    "{} http3 response send failed {:?}",
                    self.inner.trace_id(),
                    e
                );
            }
        }

        // Send response body
        let bytes_written =
            match http3_cx.send_body(&mut self.inner, response.stream_id, &response.body, true) {
                Ok(written) => written,

                Err(quiche::h3::Error::Done) => 0,

                Err(e) => {
                    error!(
                        "{} http3 response send failed {:?}",
                        self.inner.trace_id(),
                        e
                    );
                    return Ok(());
                }
            };

        // If the bytes written are less than the body,
        // store the partial response, that will be retried.
        if bytes_written < response.body.len() {
            self.partial_responses
                .insert(response.consumed_partial(bytes_written));
        }

        Ok(())
    }

    /// Handle HTTP/3 Connection Writes.
    pub fn write_http3_responses(&mut self) -> Result<(), Error> {
        self.inner.writable().for_each(|id| {
            if let Err(e) = self.write_http3_response(id) {
                error!("Failed to write HTTP/3 response: {:?}", e);
            }
        });

        Ok(())
    }

    /// Handle HTTP/3 Connection Responses.
    pub fn write_http3_response(&mut self, stream_id: u64) -> Result<(), Error> {
        if let Some(response) = self.partial_responses.0.get_mut(&stream_id) {
            let http3_cx = self.http3.as_mut().ok_or(Error::Http3ConnectionNotFound)?;

            if let Some(headers) = &mut response.headers {
                // Process Headers
                match http3_cx.send_response(&mut self.inner, stream_id, &headers, false) {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Failed to write http3 response to stream {stream_id}: {:?}",
                            e
                        );
                        return Ok(());
                    }
                }
            }

            // Set headers to None to avoid duplication.
            response.headers = None;

            let body = &response.body[response.written..];

            match http3_cx.send_body(&mut self.inner, stream_id, body, true) {
                Ok(written) => {
                    response.written += written;
                }
                // If a Done error is received, no more data can be written.
                Err(quiche::h3::Error::Done) => {}
                Err(e) => {
                    error!(
                        "Failed to write http3 response body to stream {stream_id}: {:?}",
                        e
                    );
                    self.partial_responses.remove(stream_id);
                    return Ok(());
                }
            };

            // Remove the partial response if all the data has been written.
            if response.written == response.body.len() {
                self.partial_responses.remove(stream_id);
            }
        };

        Ok(())
    }

    /// Handle HTTP/3 Connection Requests.
    pub fn read_http3_requests(&mut self) -> Result<(), Error> {
        // Keep track of the current request when parsing the body following the headers.
        let mut current_request = None;

        loop {
            let http3_cx = self.http3.as_mut().ok_or(Error::Http3ConnectionNotFound)?;

            match http3_cx.poll(&mut self.inner) {
                Ok((stream_id, quiche::h3::Event::Headers { list, has_body })) => {
                    if has_body {
                        // Event::Data should follow this event.
                        // Store the request headers in the current_request.
                        current_request = Some(RpcRequest::without_body(stream_id, list));
                    } else {
                        // If there is no body with this request, handle the request without a body.
                        self.handle_request(RpcRequest::without_body(stream_id, list))?;
                    }
                }

                Ok((stream_id, quiche::h3::Event::Data)) => {
                    info!(
                        "{} received data from stream id {}",
                        self.inner.trace_id(),
                        stream_id
                    );

                    // Get the current request and its body.
                    let request = current_request.as_mut().ok_or(Error::Http3RequestError)?;
                    // Write data to the request body.
                    let mut body = request.body.as_mut().ok_or(Error::Http3RequestError)?;

                    // NOTE: `recv_body` will return `Error::Done` when the body is fully read.
                    match http3_cx.recv_body(&mut self.inner, stream_id, &mut body) {
                        Ok(received) => {
                            info!("Received {} bytes", received);
                        }
                        Err(quiche::h3::Error::Done) => {
                            info!("Finished reading body from stream {stream_id}");
                            self.handle_request(request.clone())?;
                            current_request = None;
                        }
                        Err(e) => {
                            error!(
                                "Failed to read http3 request body from stream {stream_id}: {:?}",
                                e
                            );
                            current_request = None;
                        }
                    }
                }
                // Stream closed.
                Ok((_stream_id, quiche::h3::Event::Finished)) => (),
                // Stream reset.
                Ok((_stream_id, quiche::h3::Event::Reset { .. })) => (),
                Ok((_prioritized_element_id, quiche::h3::Event::PriorityUpdate)) => (),
                Ok((_goaway_id, quiche::h3::Event::GoAway)) => (),
                // No remaining work on the connection.
                Err(quiche::h3::Error::Done) => {
                    break;
                }
                Err(e) => {
                    error!("{} HTTP/3 error {:?}", self.inner.trace_id(), e);

                    break;
                }
            }
        }

        Ok(())
    }

    /// Suspends the inner stream while the http3 stream is being processed.
    fn suspend_stream(&mut self, stream_id: u64) -> Result<(), Error> {
        self.inner
            .stream_shutdown(stream_id, quiche::Shutdown::Read, 0)?;

        Ok(())
    }

    pub fn handle_request(&mut self, request: RpcRequest<quiche::h3::Header>) -> Result<(), Error> {
        // Suspend the inner stream while the http3 stream is being processed.
        // This is to avoid the inner stream being polled while the http3 stream is being processed.
        self.suspend_stream(request.stream_id)?;

        // Process the inner requests, e.g. RPC requests.
        let response = self.handle_inner_request(request)?;

        // Send response to the http3 stream.
        self.handle_response(response)?;

        Ok(())
    }

    /// Handle the inner RPC request.
    /// The route is determined by the request headers.
    pub fn handle_inner_request(
        &mut self,
        request: RpcRequest<quiche::h3::Header>,
    ) -> Result<RpcResponse<quiche::h3::Header>, Error> {
        info!("Received Request: {:?}", request);

        error!("RpcServer::handle_inner_request not implemented");
        unimplemented!("RpcServer::handle_inner_request not implemented")
    }
}

pub type RpcConnectionsLock<'a> = std::sync::MutexGuard<
    'a,
    std::collections::HashMap<
        quiche::ConnectionId<'static>,
        RpcConnection<quiche::Connection, quiche::h3::Connection, quiche::h3::Header>,
    >,
>;

// pub type RpcConnectionsLockError<'a> = std::sync::PoisonError<
//     std::sync::MutexGuard<
//         'a,
//         std::collections::HashMap<
//             quiche::ConnectionId<'static>,
//             RpcConnection<quiche::Connection, quiche::h3::Connection, quiche::h3::Header>,
//         >,
//     >,
// >;

pub struct RpcConnections<Id, Conn, Http3Conn, Header>(
    Mutex<HashMap<Id, RpcConnection<Conn, Http3Conn, Header>>>,
);

impl
    RpcConnections<
        quiche::ConnectionId<'static>,
        quiche::Connection,
        quiche::h3::Connection,
        quiche::h3::Header,
    >
{
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn lock(&self) -> Result<RpcConnectionsLock, Error> {
        self.0.lock().map_err(|e| Error::ConnectionsLockPoisioned)
    }

    pub fn timeout(&self) -> Result<(), Error> {
        if let Ok(mut lock) = self.lock() {
            lock.iter_mut().for_each(|(_, c)| c.inner.on_timeout());
        }

        Ok(())
    }
}

/// RPC Server.
pub struct RpcServer<Id, Conn, QConfig: Clone, Http3Conn, Header> {
    socket: UdpSocket,
    poll: Poll,
    events: Events,
    quic_config: QuicConfig<QConfig>,
    connections: RpcConnections<Id, Conn, Http3Conn, Header>,
    hmac_key: ring::hmac::Key,
}

impl
    RpcServer<
        quiche::ConnectionId<'static>,
        quiche::Connection,
        quiche::Config,
        quiche::h3::Connection,
        quiche::h3::Header,
    >
{
    /// Bind to a socket address, listening on UDP.
    pub fn bind(
        socket_address: impl TryInto<SocketAddr>,
        quic_config: Option<QuicConfig<quiche::Config>>,
    ) -> Result<Self, Error> {
        let socket_address = socket_address
            .try_into()
            .map_err(|_| Error::InvalidSocketAddress)?;

        let mut socket = UdpSocket::bind(socket_address)?;

        // Register the socket with the poll.
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut socket, DEFAULT_SOCKET, Interest::READABLE)?;

        let events = Events::with_capacity(DEFAULT_EVENTS_CAPACITY);
        let connections = RpcConnections::new();
        let hmac_key = Self::generate_hmac_key()?;

        let quic_config = quic_config
            .map(|c| Ok(c))
            .unwrap_or_else(|| QuicConfig::development(TlsOptions::None))?;

        Ok(Self {
            socket,
            poll,
            events,
            connections,
            hmac_key,
            quic_config,
        })
    }

    /// Return the local address of the server.
    pub fn address(&self) -> Result<SocketAddr, Error> {
        Ok(self.socket.local_addr()?)
    }

    /// Start the RPC server.
    pub async fn start(&mut self) -> Result<(), Error> {
        let mut input_buffer = Vec::from([0; MAX_UDP_PAYLOAD_SIZE]);
        let mut output_buffer = Vec::from([0; MAX_QUIC_DATAGRAM_SIZE]);

        loop {
            self.poll
                .poll(&mut self.events, Some(DEFAULT_POLL_WINDOW))?;

            'incoming: loop {
                // Empty events indicates the connections have timed out,
                // and failed to send a keep-alive packet.
                // Break the incoming loop and proceed to the outer loop,
                // to re-poll the socket.
                if self.events.is_empty() {
                    self.connections.timeout()?;

                    break 'incoming;
                }

                // Check for incoming events.
                debug!("Check incoming packets");

                let (bytes_received, remote_socket_addr) =
                    match self.socket.recv_from(&mut input_buffer) {
                        Ok(received) => {
                            debug!("Received Incoming Bytes");
                            received
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                debug!("Receiving Incoming Packets Would Block");

                                break 'incoming;
                            }

                            return Err(Error::Io(e));
                        }
                    };

                debug!("Received {bytes_received} from {remote_socket_addr}");

                let packet_buffer = &mut input_buffer[..bytes_received];

                debug!("Packet Buffer: {:?}", packet_buffer);

                let quic_header =
                    match quiche::Header::from_slice(packet_buffer, quiche::MAX_CONN_ID_LEN) {
                        Ok(header) => header,
                        Err(e) => {
                            error!("Failed to parse QUIC header: {}", e);

                            // Finish the incoming loop early.
                            continue 'incoming;
                        }
                    };

                debug!("Quic Packet Header {:?}", quic_header);

                // Retirieve the unverified connection id from the quic header.
                let server_id = self.server_connection_id(&quic_header)?;
                let mut source_id = [0; quiche::MAX_CONN_ID_LEN];
                source_id.copy_from_slice(&server_id);
                let source_id = quiche::ConnectionId::from_ref(&source_id);

                // Get a lock on the connections.
                let mut connections = self.connections()?;

                let connection = match connections.get_mut(&quic_header.dcid) {
                    // Connection has already been verified and is stored in the connections map.
                    Some(connection) => connection,
                    // Need to establish a new verified connection id.
                    None => {
                        debug!("Establishing new connection");

                        // If the header type is not `Initial`, then the connection is not new,
                        // and we should ignore it.
                        if quic_header.ty != quiche::Type::Initial {
                            continue 'incoming;
                        }

                        // Negotiate the quic version, if the header version is not supported.
                        if !quiche::version_is_supported(quic_header.version) {
                            debug!("Negotiating Version");

                            let bytes_written = quiche::negotiate_version(
                                &quic_header.scid,
                                &quic_header.dcid,
                                &mut output_buffer,
                            )?;

                            let data = &output_buffer[..bytes_written];

                            // Notify the sender that the version is not supported,
                            // and negotiate a new version.
                            match self.socket.send_to(data, remote_socket_addr) {
                                Ok(_) => {
                                    debug!("Sent version negotiation packet");
                                }
                                Err(e) => match e.kind() {
                                    std::io::ErrorKind::WouldBlock => break,
                                    _ => {
                                        error!("Failed to send version negotiation packet: {}", e);
                                    }
                                },
                            };

                            continue 'incoming;
                        }

                        // Return the Original Destination Connection ID (ODCID) from the token.
                        // This corresponds to the client's initial destination connection id (DCID) used.
                        let odcid = match self.original_destination_id(
                            &quic_header,
                            &remote_socket_addr,
                            &source_id.clone(),
                            &mut output_buffer,
                        ) {
                            Ok(Some(odcid)) => odcid,
                            Err(e) => {
                                error!("Failed to retrieve ODCID: {}", e);

                                // Finish the incoming loop early.
                                continue 'incoming;
                            }
                            _ => {
                                warn!("ODCID not present, dropping incoming packet.");

                                // Drop the packet if the ODCID is not present.
                                continue 'incoming;
                            }
                        };

                        let connection = quiche::accept(
                            &quic_header.dcid.clone(),
                            Some(&odcid),
                            self.address()?,
                            remote_socket_addr,
                            // Make a mutable copy of the quic config.
                            &mut self.quic_config.clone().into(),
                        )?;

                        // Accept the connection.
                        connections.insert(quic_header.dcid.clone(), connection.into());

                        info!("Accepted New Connection: {:?}", server_id);

                        // Return a mutable reference to the connection.
                        connections
                            .get_mut(&quic_header.dcid)
                            .ok_or(Error::InvalidConnection)?
                    }
                };

                // Read the packet into the connection.
                let read = match connection.inner.recv(
                    packet_buffer,
                    quiche::RecvInfo {
                        to: self.address()?,
                        from: remote_socket_addr,
                    },
                ) {
                    Ok(read) => read,
                    Err(e) => {
                        error!(
                            "{} Failed to read packet into connection: {}",
                            connection.inner.trace_id(),
                            e
                        );

                        continue 'incoming;
                    }
                };

                debug!("Read Packet into Connection: {:?}", read);

                // Create a new HTTP/3 Connection.
                if !connection.is_http3_established() {
                    info!("QUIC Handshake handled, creating HTTP/3 Handshake");

                    let http3_cx = match quiche::h3::Connection::with_transport(
                        &mut connection.inner,
                        // TODO: Make a mutable copy of the global http3 config.
                        &mut quiche::h3::Config::new()?,
                    ) {
                        Ok(connection) => connection,
                        Err(e) => {
                            error!("Failed to create HTTP/3 connection: {}", e);
                            continue 'incoming;
                        }
                    };

                    connection.set_http3(http3_cx);
                }

                // Handle the HTTP/3 connection.
                if connection.http3().is_some() {
                    debug!("Handling HTTP/3 Connection");

                    // Write HTTP/3 responses.
                    connection.write_http3_responses()?;

                    // Read HTTP/3 requests.
                    connection.read_http3_requests()?;
                }
            } // End of incoming loop.

            // Process outgoing quic packets.
            let mut connections = self.connections()?;
            for (connection_id, connection) in connections.iter_mut() {
                debug!(
                    "Processing Outgoing Quic Packets for Connection ID: {:?}",
                    connection_id
                );
                loop {
                    let (bytes_written, recipient) = match connection.inner.send(&mut output_buffer)
                    {
                        Ok(sent) => sent,

                        Err(quiche::Error::Done) => {
                            debug!("{} done writing", connection.inner.trace_id());
                            break;
                        }

                        Err(e) => {
                            error!("{} send failed: {:?}", connection.inner.trace_id(), e);

                            connection.inner.close(false, 0x1, b"fail").ok();
                            break;
                        }
                    };

                    if let Err(e) = self
                        .socket
                        .send_to(&output_buffer[..bytes_written], recipient.to)
                    {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            debug!("Sending would block");
                            break;
                        }

                        error!(
                            "Failed to send packet to connection {}: {}",
                            connection.inner.trace_id(),
                            e
                        );

                        return Err(Error::Io(e));
                    }

                    info!(
                        "{} written {} bytes",
                        connection.inner.trace_id(),
                        bytes_written
                    );
                }
            }

            // Clean up closed connections.
            connections.retain(|_, conn| {
                // Forget the connection if it is closed.
                if conn.inner.is_closed() {
                    debug!("Connection {} closed", conn.inner.trace_id());
                    return false;
                }

                // Retain open connections.
                true
            });
        }
    }

    /// Generation an HMAC key used as the seed for establishing connection ids.
    fn generate_hmac_key() -> Result<ring::hmac::Key, Error> {
        ring::hmac::Key::generate(ring::hmac::HMAC_SHA256, &ring::rand::SystemRandom::new())
            .map_err(|_| Error::HmacKeyGeneration)
    }

    /// Return the socket address as IP address bytes.
    fn socket_addr_bytes(addr: &SocketAddr) -> Vec<u8> {
        match addr.ip() {
            std::net::IpAddr::V4(a) => a.octets().to_vec(),
            std::net::IpAddr::V6(a) => a.octets().to_vec(),
        }
    }

    /// Generate a token for the connection.
    fn generate_auth_token<'a>(
        &self,
        id: &quiche::ConnectionId<'a>,
        remote_socket_addr: &SocketAddr,
    ) -> Result<Vec<u8>, Error> {
        let mut token = DEFAULT_TOKEN_PREFIX.to_vec();

        let id = id.as_ref();
        token.push(id.len() as u8);
        token.extend_from_slice(id);

        let addr_bytes = Self::socket_addr_bytes(remote_socket_addr);
        token.push(addr_bytes.len() as u8);
        token.extend_from_slice(&addr_bytes);

        debug!("Sign Token Data: {:?}", token);

        // Sign the token with the HMAC key.
        let signature = ring::hmac::sign(&self.hmac_key, &token);
        let signature = signature.as_ref();

        debug!("Generate Auth Token Signature: {:?}", signature);

        token.push(signature.len() as u8);
        token.extend_from_slice(signature);

        Ok(token)
    }

    /// Validate a token, returning the connection ID if the token is valid.
    fn validate_auth_token(
        &self,
        token: &mut Vec<u8>,
        remote_socket_addr: &SocketAddr,
    ) -> Result<quiche::ConnectionId, Error> {
        // Drain the token prefix.
        let mut data = token
            .drain(..DEFAULT_TOKEN_PREFIX.len())
            .collect::<Vec<u8>>();

        let id_len = token.remove(0) as usize;
        let id = token.drain(..id_len).collect::<Vec<u8>>();
        let conn_id = quiche::ConnectionId::from_vec(id.clone());
        data.push(id_len as u8);
        data.extend_from_slice(id.as_slice());

        let addr_len = token.remove(0) as usize;
        let addr = token.drain(..addr_len).collect::<Vec<u8>>();
        data.push(addr_len as u8);
        data.extend_from_slice(addr.as_slice());

        if addr != Self::socket_addr_bytes(remote_socket_addr) {
            return Err(Error::InvalidAuthToken);
        }

        debug!("Verify Token Data: {:?}", data);

        let signature_len = token.remove(0) as usize;
        let signature = token.drain(..signature_len).collect::<Vec<u8>>();

        debug!("Verify Auth Token Signature: {:?}", signature);

        // Verify the token signature.
        ring::hmac::verify(&self.hmac_key, &data, &signature).map_err(|e| {
            error!("Failed to verify token signature: {:?}", e);
            Error::InvalidAuthToken
        })?;

        // Return the verified connection ID.
        Ok(conn_id)
    }

    /// Create a new connection ID using the connection seed ID.
    fn server_connection_id(&self, header: &quiche::Header) -> Result<quiche::ConnectionId, Error> {
        // Return the connection ID from the destination connection ID (DCID).
        let tag = ring::hmac::sign(&self.hmac_key, &header.dcid);
        tag.as_ref()[..quiche::MAX_CONN_ID_LEN]
            .to_vec()
            .try_into()
            .map_err(|_| Error::ConnectionIdGeneration)
    }

    fn connections(&self) -> Result<RpcConnectionsLock, Error> {
        self.connections.lock()
    }

    /// Parse the Original Destination Connection ID (ODCID) from the token.
    fn original_destination_id(
        &self,
        header: &quiche::Header,
        remote_socket_addr: &SocketAddr,
        new_scid: &quiche::ConnectionId,
        mut output_buffer: &mut Vec<u8>,
    ) -> Result<Option<quiche::ConnectionId>, Error> {
        let odcid = match header
            .token
            .clone()
            // Check if the token is empty, and filter if so.
            .and_then(|t| if t.is_empty() { None } else { Some(t) })
        {
            // Verify the token and return the original destination connection id.
            Some(mut token) => Some(self.validate_auth_token(&mut token, &remote_socket_addr)?),
            // Initial connection packet should include
            // a token. Attempt to retry the connection
            // if the token is missing.
            None => {
                debug!("No token provided, generating a new one");

                // Generate a signed authentication token.
                let token = self.generate_auth_token(&header.dcid, &remote_socket_addr)?;

                debug!("Auth Token: {:?}", token);

                let bytes_written = quiche::retry(
                    &header.scid,
                    &header.dcid,
                    &new_scid,
                    token.as_slice(),
                    PROTOCOL_VERSION,
                    &mut output_buffer,
                )?;

                debug!(
                    "Retry packet written: {:?}",
                    &output_buffer[..bytes_written]
                );

                let data = &output_buffer[..bytes_written];

                self.socket.send_to(data, remote_socket_addr.to_owned())?;

                debug!("Sent re-try data to client");

                None
            }
        };

        Ok(odcid)
    }

    // Utility method for closing a connection with a transport error code.
    pub fn close_connection(
        connection: &mut quiche::Connection,
        error_code: TransportErrorCode,
    ) -> Result<(), Error> {
        warn!("Closing connection with error code: {:?}", error_code);

        let (inform_peer, code, reason) = error_code.into_parts();

        connection.close(inform_peer, code, reason.as_bytes())?;

        Ok(())
    }
}
