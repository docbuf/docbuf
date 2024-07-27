use crate::{
    close_connection,
    error::Error,
    quic::{QuicConfig, TlsOptions, TransportErrorCode, MAX_QUIC_DATAGRAM_SIZE},
    server::MAX_UDP_PAYLOAD_SIZE,
    PartialRpcResponse, PartialRpcResponses, RpcRequest, RpcRequestSender, RpcResponse,
};

use std::time::Duration;
use std::{net::SocketAddr, sync::mpsc::TryRecvError, thread::JoinHandle};

use mio::{net::UdpSocket, Events, Poll, Token};
use quiche::SendInfo;
use ring::rand::*;
use tracing::{debug, error, info, warn};

/// Default socket token.
const DEFAULT_SOCKET: Token = Token(0);

#[derive(Debug)]
pub struct RpcClient {
    sender: RpcRequestSender,
    daemon: JoinHandle<Result<(), Error>>,
}

impl RpcClient {
    pub fn send(&self, request: RpcRequest) -> Result<RpcResponse, Error> {
        let sender = self.sender.clone();
        let response = std::thread::spawn(move || {
            debug!("Spawning RPC Client Receiver Response Thread");
            let (req_tx, req_rx) = RpcResponse::oneshot();

            debug!("Sending RPC Client Request");
            sender.send((request, req_tx))?;

            loop {
                match req_rx.try_recv() {
                    Ok(response) => {
                        debug!("Received RPC Client Response.");
                        return Ok(response);
                    }
                    Err(TryRecvError::Empty) => {
                        // error!("Received Empty response");
                        continue;
                    }
                    Err(TryRecvError::Disconnected) => {
                        error!("Received Disconnected response");
                        return Err(Error::RpcClientReceiverDisconnected);
                    }
                }
            }
        });

        response
            .join()
            .map_err(|_| Error::RpcClientReceiverThreadFailed)?
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
    pub fn connect(
        peer_addr: impl Into<SocketAddr>,
        server_name: Option<&str>,
        config: Option<QuicConfig<quiche::Config>>,
    ) -> Result<Self, Error> {
        let mut socket = UdpSocket::bind("[::]:0".parse()?)?;

        let mut queue = Poll::new()?;
        let mut events = Events::with_capacity(1024);

        // Register the default socket token.
        queue
            .registry()
            .register(&mut socket, DEFAULT_SOCKET, mio::Interest::READABLE)?;

        let mut config = config
            .map(|c| Ok(c))
            .unwrap_or_else(|| QuicConfig::development(TlsOptions::None))?;

        // let mut send_test_request = true;

        let mut input_buffer = Vec::from([0; MAX_UDP_PAYLOAD_SIZE]);
        let mut output_buffer = Vec::from([0; MAX_QUIC_DATAGRAM_SIZE]);

        let mut id = [0; quiche::MAX_CONN_ID_LEN];
        let connection_id = Self::new_connection_id(&mut id)?;

        let local_addr = socket.local_addr()?;
        let peer_addr = peer_addr.into();

        info!("Connecting to {:?} from {:?}", peer_addr, local_addr);

        // Create the Quic connection.
        let mut connection = quiche::connect(
            server_name,
            &connection_id,
            local_addr,
            peer_addr,
            &mut config.0,
        )?;

        info!("Connection established");

        let mut http3_cx = None;
        let http3_config = quiche::h3::Config::new()?;

        // Complete the connection handshake.
        let (bytes_written, SendInfo { to, .. }) = connection.send(&mut output_buffer)?;

        debug!("Preparing to send {} bytes", bytes_written);

        while let Err(e) = socket.send_to(&output_buffer[..bytes_written], to) {
            // Re-try if the request would block.
            if e.kind() == std::io::ErrorKind::WouldBlock {
                debug!("Sending would block");
                continue;
            }

            return Err(Error::Io(e));
        }

        let (req_tx, req_rx) = RpcRequest::channel();

        // Keep track of partial responses.
        let mut partial_responses = PartialRpcResponses::new();
        let mut sync_senders = RpcResponse::sync_senders();

        let daemon = std::thread::spawn(move || {
            'main: loop {
                let timeout = Some(Duration::from_nanos(1));
                // NOTE: `connection.timeout()` caused the poll
                // to block for ~5s, using an explicit timeout
                // to avoid this issue.
                queue.poll(&mut events, timeout)?;

                // Check for incoming packets
                'incoming: loop {
                    // Break if no events are found.
                    if events.is_empty() {
                        connection.timeout();

                        // debug!("No events found");

                        break 'incoming;
                    }

                    // debug!("Found Event");

                    let (bytes_received, remote_socket_addr) =
                        match socket.recv_from(&mut input_buffer) {
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

                    match connection.recv(
                        &mut input_buffer[..bytes_received],
                        quiche::RecvInfo {
                            to: socket.local_addr()?,
                            from: remote_socket_addr,
                        },
                    ) {
                        Ok(bytes_read) => {
                            // Received QUIC packet
                            debug!("Received {bytes_read} bytes from remote socket");
                        }
                        Err(e) => {
                            error!("Recieved Error When Reading from Remote Socket: {:?}", e);
                            continue 'incoming;
                        }
                    };
                } // end of 'incoming loop

                if connection.is_closed() {
                    debug!("Connection closed");
                    break;
                }

                // Handle http3 connection
                if http3_cx.is_none() && connection.is_established() {
                    debug!("Establishing HTTP/3 connection");
                    http3_cx = Some(quiche::h3::Connection::with_transport(
                        &mut connection,
                        &http3_config,
                    )?);
                }

                if let Some(http3_cx) = &mut http3_cx {
                    debug!("Processing HTTP/3 Requests");
                    // Processing Outgoing Requests to the Server over HTTP/3
                    match req_rx.try_recv() {
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => {
                            error!("Received Disconnected response");
                            break;
                        }
                        Ok((mut request, res_tx)) => {
                            debug!("Attempting to Send HTTP/3 Request");
                            let bodyless = request.body.is_none();

                            let content_length = request.headers.content_length().unwrap_or(0);

                            let stream_id = http3_cx.send_request(
                                &mut connection,
                                &request.headers,
                                bodyless,
                            )?;

                            // Set the request stream id.
                            request.stream_id = stream_id;

                            debug!("Sending Request for Stream ID: {stream_id}");

                            // Save the response sender for this stream.
                            sync_senders.insert(request.stream_id, res_tx);

                            if let Some(body) = &mut request.body {
                                match http3_cx.send_body(
                                    &mut connection,
                                    request.stream_id,
                                    body,
                                    false,
                                ) {
                                    Ok(written) => {
                                        info!("Wrote {written} of {content_length} bytes to stream {stream_id}");
                                    }
                                    Err(quiche::h3::Error::Done) => {
                                        debug!(
                                            "{} done writing http/3 request stream {:?}",
                                            connection.trace_id(),
                                            request.stream_id
                                        );
                                    }

                                    Err(e) => {
                                        error!("{} send failed: {:?}", connection.trace_id(), e);

                                        close_connection(
                                            &mut connection,
                                            TransportErrorCode::InternalError,
                                        )?;

                                        break 'main;
                                    }
                                }
                            }
                        }
                    }

                    // Listen for HTTP/3 incoming events.
                    // Process HTTP/3 events.
                    loop {
                        debug!("Listening for HTTP/3 Responses");
                        match http3_cx.poll(&mut connection) {
                            Ok((stream_id, quiche::h3::Event::Headers { list, has_body })) => {
                                debug!("Received headers {list:?} on stream id {stream_id}");

                                if has_body {
                                    // Create a new partial response.
                                    partial_responses
                                        .insert(PartialRpcResponse::new(stream_id, list));
                                } else {
                                    // Bodyless response returns immediately.
                                    sync_senders.get_mut(&stream_id).and_then(|res_tx| {
                                        res_tx
                                            .send(RpcResponse::with_empty_body(stream_id, list))
                                            .ok()
                                    });
                                    break;
                                }
                            }

                            Ok((stream_id, quiche::h3::Event::Data)) => {
                                info!("Received HTTP/3 Data Event");
                                match partial_responses.remove(&stream_id) {
                                    Some(mut partial_response) => loop {
                                        match http3_cx.recv_body(
                                            &mut connection,
                                            stream_id,
                                            &mut partial_response.body,
                                        ) {
                                            Ok(read) => {
                                                partial_response.written += read;
                                                debug!(
                                                    "Received {} bytes of response data on stream {stream_id}",
                                                    partial_response.written
                                                );
                                            }
                                            Err(quiche::h3::Error::Done) => {
                                                let response: RpcResponse = partial_response.into();

                                                info!("Finished Reading Response");
                                                // No more data to read. Send the response.
                                                let res_tx = sync_senders
                                                    // Note: if we remove the res_tx, the connection will
                                                    // disconnect. Cleanup after the response is sent.
                                                    .remove(&stream_id)
                                                    .ok_or(Error::InternalError)?;

                                                info!("Sending Response");

                                                res_tx.send(response).map_err(|e| {
                                                    error!("Error when sending response: {:?}", e);
                                                    Error::InternalError
                                                })?;

                                                break;
                                            }
                                            Err(e) => {
                                                error!(
                                                    "{} recv failed: {:?}",
                                                    connection.trace_id(),
                                                    e
                                                );

                                                close_connection(
                                                    &mut connection,
                                                    TransportErrorCode::InternalError,
                                                )?;

                                                break;
                                            }
                                        }
                                    },
                                    None => {
                                        warn!("received data for unknown stream id {}", stream_id);

                                        // Continue to the next event.
                                        continue;
                                    }
                                };
                            }

                            Ok((stream_id, quiche::h3::Event::Finished)) => {
                                warn!("HTTP/3 Stream {stream_id} Finished");

                                // sync_senders.remove(&stream_id);

                                break;

                                // close_connection(
                                //     &mut connection,
                                //     TransportErrorCode::ApplicationError(
                                //         "Connection Finished".to_string(),
                                //     ),
                                // )?;
                            }

                            Ok((_stream_id, quiche::h3::Event::Reset(e))) => {
                                error!("request was reset by peer with {}, closing...", e);

                                close_connection(
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
                                // debug!("{} done processing HTTP/3 events", connection.trace_id());
                                break;
                            }

                            Err(e) => {
                                error!("HTTP/3 processing failed: {:?}", e);

                                break;
                            }
                        }
                    } // End of HTTP/3 read event loop.
                }

                debug!("Processing Outgoing QUIC Packets");

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

                    if let Err(e) = socket.send_to(&output_buffer[..written], recipient.to) {
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
                debug!("End of Main Loop");
            } // End of main loop.

            Ok::<(), Error>(())
        });

        Ok(Self {
            daemon,
            sender: req_tx,
        })
    }
}
