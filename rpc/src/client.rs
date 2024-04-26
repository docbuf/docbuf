use std::net::SocketAddr;

use crate::{
    error::Error,
    quic::{QuicConfig, TlsOptions, TransportErrorCode, MAX_QUIC_DATAGRAM_SIZE},
    server::MAX_UDP_PAYLOAD_SIZE,
    RpcHeader,
};

use mio::{net::UdpSocket, Events, Poll, Token};
use ring::rand::*;
use tracing::{debug, error, info, warn};

/// Default socket token.
const DEFAULT_SOCKET: Token = Token(0);

pub struct RpcClient<Conn, Config: Clone> {
    socket: UdpSocket,
    queue: Poll,
    events: Events,
    http3_cx: Option<Conn>,
    config: QuicConfig<Config>,
}

impl RpcClient<quiche::h3::Connection, quiche::Config> {
    pub fn new(
        local_addr: Option<impl Into<SocketAddr>>,
        config: Option<QuicConfig<quiche::Config>>,
    ) -> Result<Self, Error> {
        let local_addr = local_addr
            .map(|addr| Ok::<SocketAddr, Error>(addr.into()))
            // Otherwise use a random ipv6 local port.
            .unwrap_or_else(|| Ok("[::]:0".parse()?))?;

        let mut socket = UdpSocket::bind(local_addr)?;

        let queue = Poll::new()?;
        let events = Events::with_capacity(1024);

        // Register the default socket token.
        queue
            .registry()
            .register(&mut socket, DEFAULT_SOCKET, mio::Interest::READABLE)?;

        let config = config
            .map(|c| Ok(c))
            .unwrap_or_else(|| QuicConfig::development(TlsOptions::None))?;

        Ok(Self {
            socket,
            queue,
            events,
            http3_cx: None,
            config,
        })
    }

    fn new_connection_id<'a>(
        buffer: &'a mut [u8; quiche::MAX_CONN_ID_LEN],
    ) -> Result<quiche::ConnectionId<'a>, Error> {
        SystemRandom::new()
            .fill(&mut buffer[..])
            .map_err(|_| Error::InvalidConnectionId)?;

        let id = quiche::ConnectionId::from_ref(buffer.as_slice());

        Ok(id)
    }

    /// Starts a connection to a remote server.
    pub async fn connect(
        &mut self,
        peer_addr: impl Into<SocketAddr>,
        server_name: Option<&str>,
    ) -> Result<(), Error> {
        let mut send_test_request = true;

        let mut input_buffer = Vec::from([0; MAX_UDP_PAYLOAD_SIZE]);
        let mut output_buffer = Vec::from([0; MAX_QUIC_DATAGRAM_SIZE]);

        let mut id = [0; quiche::MAX_CONN_ID_LEN];
        let connection_id = Self::new_connection_id(&mut id)?;

        let local_addr = self.socket.local_addr()?;
        let peer_addr = peer_addr.into();

        info!("Connecting to {:?} from {:?}", peer_addr, local_addr);

        // Create the Quic connection.
        let mut connection = quiche::connect(
            server_name,
            &connection_id,
            local_addr,
            peer_addr,
            &mut self.config.0,
        )?;

        info!("Connection established");

        let http3_config = quiche::h3::Config::new()?;

        // Complete the connection handshake.
        let (bytes_written, remote_socket_address) = connection.send(&mut output_buffer)?;

        debug!("Preparing to send {} bytes", bytes_written);

        while let Err(e) = self
            .socket
            .send_to(&output_buffer[..bytes_written], remote_socket_address.to)
        {
            // Re-try if the request would block.
            if e.kind() == std::io::ErrorKind::WouldBlock {
                debug!("Sending would block");
                continue;
            }

            return Err(Error::Io(e));
        }

        debug!("Entering Main Loop");

        loop {
            self.queue.poll(&mut self.events, connection.timeout())?;

            // Check for incoming packets
            'incoming: loop {
                // Break if no events are found.
                if self.events.is_empty() {
                    connection.timeout();

                    break 'incoming;
                }

                debug!("Found Event");

                let (bytes_received, remote_socket_addr) =
                    match self.socket.recv_from(&mut input_buffer) {
                        Ok(received) => received,
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                debug!("Receiving incoming UDP packets would block");
                                break 'incoming;
                            }

                            return Err(Error::Io(e));
                        }
                    };

                info!(
                    "Received {} bytes from {:?}",
                    bytes_received, remote_socket_addr
                );

                let read = match connection.recv(
                    &mut input_buffer[..bytes_received],
                    quiche::RecvInfo {
                        to: self.socket.local_addr()?,
                        from: remote_socket_addr,
                    },
                ) {
                    Ok(read) => read,
                    Err(e) => {
                        error!("Recieved Error When Reading from Remote Socket: {:?}", e);
                        continue 'incoming;
                    }
                };

                // handle the read data
                // TODO: Handle the read data.

                debug!("Received {read} bytes from remote socket");
            } // end of 'incoming loop

            if connection.is_closed() {
                debug!("Connection closed");
                break;
            }

            // Handle http3 connection
            if self.http3_cx.is_none() && connection.is_established() {
                debug!("Establishing HTTP/3 connection");
                self.http3_cx = Some(quiche::h3::Connection::with_transport(
                    &mut connection,
                    &http3_config,
                )?);
            }

            if let Some(http3_cx) = &mut self.http3_cx {
                // TODO: Handle HTTP/3 Requests.

                // Send a test request.
                if send_test_request {
                    info!("Sending test request");

                    let headers = vec![
                        RpcHeader::new(b":method", b"POST"),
                        RpcHeader::new(b":scheme", b"http"),
                        RpcHeader::new(b":authority", b"localhost"),
                        RpcHeader::new(b":path", b"/complex/save_document"),
                        RpcHeader::new(b"content-type", b"application/docbuf+rpc"),
                        RpcHeader::new(b"user-agent", b"docbuf-rpc"),
                    ];
                    let bodyless = true;
                    http3_cx.send_request(&mut connection, &headers, bodyless)?;

                    send_test_request = false;
                }
            }

            // Listen for HTTP/3 events.
            if let Some(http3_cx) = &mut self.http3_cx {
                // Process HTTP/3 events.
                loop {
                    match http3_cx.poll(&mut connection) {
                        Ok((stream_id, quiche::h3::Event::Headers { list, .. })) => {
                            debug!("got response headers {:?} on stream id {}", list, stream_id);
                        }

                        Ok((stream_id, quiche::h3::Event::Data)) => {
                            while let Ok(read) =
                                http3_cx.recv_body(&mut connection, stream_id, &mut input_buffer)
                            {
                                debug!(
                                    "got {} bytes of response data on stream {}",
                                    read, stream_id
                                );

                                debug!("{}", unsafe {
                                    std::str::from_utf8_unchecked(&input_buffer[..read])
                                });
                            }
                        }

                        Ok((_stream_id, quiche::h3::Event::Finished)) => {
                            debug!("HTTP/3 Connection Finished");

                            Self::close_connection(
                                &mut connection,
                                TransportErrorCode::ApplicationError(
                                    "Connection Finished".to_string(),
                                ),
                            )?;
                        }

                        Ok((_stream_id, quiche::h3::Event::Reset(e))) => {
                            error!("request was reset by peer with {}, closing...", e);

                            Self::close_connection(
                                &mut connection,
                                TransportErrorCode::ApplicationError(
                                    "Connection Reset".to_string(),
                                ),
                            )?;
                        }

                        Ok((_, quiche::h3::Event::PriorityUpdate)) => {
                            warn!("ignoring HTTP/3 priority update");
                        }

                        Ok((goaway_id, quiche::h3::Event::GoAway)) => {
                            warn!("GOAWAY id={}", goaway_id);
                        }

                        Err(quiche::h3::Error::Done) => {
                            break;
                        }

                        Err(e) => {
                            error!("HTTP/3 processing failed: {:?}", e);

                            break;
                        }
                    }
                } // End of HTTP/3 read event loop.
            }

            // Outgoing Quic Packets;
            'outgoing: loop {
                let (written, recipient) = match connection.send(&mut output_buffer) {
                    Ok(sent) => sent,
                    Err(quiche::Error::Done) => {
                        break 'outgoing;
                    }
                    Err(_) => {
                        error!("Error Writing to Remote Socket");
                        break 'outgoing;
                    }
                };

                if let Err(e) = self.socket.send_to(&output_buffer[..written], recipient.to) {
                    // Re-try if the request would block.
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        debug!("send() would block");
                        break 'outgoing;
                    }

                    return Err(Error::Io(e));
                }
            } // End of outgoing loop.

            // Lastly, clean up the connection if it is closed.
            if connection.is_closed() {
                debug!("Connection Closed");
                break;
            }
        } // End of main loop.

        Ok(())
    }

    // Utility method for closing a connection with a transport error code.
    pub fn close_connection(
        connection: &mut quiche::Connection,
        error_code: TransportErrorCode,
    ) -> Result<(), Error> {
        let (inform_peer, code, reason) = error_code.into_parts();

        connection.close(inform_peer, code, reason.as_bytes())?;

        Ok(())
    }
}
