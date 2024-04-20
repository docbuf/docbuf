use crate::error::Error;
// use crate::http3::Http3Config;
use crate::quic::{QuicConfig, TlsOptions, TransportErrorCode, MAX_QUIC_DATAGRAM_SIZE};
use crate::{connections::*, RpcRequest, RpcResponse};

use std::net::SocketAddr;
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

/// RPC Server.
pub struct RpcServer<Id, Conn, QConfig: Clone, Http3Conn, Header> {
    socket: UdpSocket,
    queue: Poll,
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
        let queue = Poll::new()?;
        queue
            .registry()
            .register(&mut socket, DEFAULT_SOCKET, Interest::READABLE)?;

        let events = Events::with_capacity(DEFAULT_EVENTS_CAPACITY);
        let connections = RpcConnections::new();
        let hmac_key = Self::generate_hmac_key()?;

        let quic_config = quic_config
            .map(|c| Ok(c))
            .unwrap_or_else(|| QuicConfig::development(TlsOptions::None))?;

        Ok(Self {
            socket,
            queue,
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

        // Create a channel for sending and receiving requests.
        let (req_tx, req_rx) = RpcRequest::channel();

        // Process Requests on a separate thread.
        // Join to main thread prior to returning.
        let request_processor = std::thread::spawn(move || {
            // Wait for incoming requests.
            while let Ok((request, res_tx)) = req_rx.recv() {
                info!("Processing Request: {:?}", request);

                todo!("Add the Request Processor Here");

                // Send the response back to the client.
                res_tx.send(RpcResponse {
                    stream_id: request.stream_id,
                    headers: request.headers,
                    body: Vec::new(),
                })?;
            }

            Ok::<_, Error>(())
        });

        // Enter main event loop.
        loop {
            self.queue
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

                            // Wait for request processor to finish.
                            request_processor.join().map_err(|e| {
                                error!("Failed to join request processor");
                                Error::ThreadJoinError(e)
                            })??;

                            error!("Failed to receive incoming packet from UDP socket: {}", e);

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
                    connection.read_http3_requests(&req_tx)?;
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

                            connection.close(TransportErrorCode::InternalError)?;

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

                        // Wait to finish processing the requests.
                        request_processor.join().map_err(|e| {
                            error!("Failed to join request processor");
                            Error::ThreadJoinError(e)
                        })??;

                        error!("Failed to send outgoing packet from UDP socket: {}", e);

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
