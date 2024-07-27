use crate::{
    Error, PartialRpcRequests, PartialRpcResponses, RpcHeaders, RpcRequest, RpcRequestSender,
    RpcResponse, TransportErrorCode,
};

use std::collections::HashMap;
use std::sync::Mutex;

use tracing::{debug, error, info, warn};

pub struct RpcConnection<Conn, Http3Conn> {
    pub(crate) inner: Conn,
    pub(crate) http3: Option<Http3Conn>,
    pub(crate) partial_responses: PartialRpcResponses,
    pub(crate) partial_requests: PartialRpcRequests,
}

impl From<quiche::Connection> for RpcConnection<quiche::Connection, quiche::h3::Connection> {
    fn from(inner: quiche::Connection) -> Self {
        Self::new(inner)
    }
}

impl RpcConnection<quiche::Connection, quiche::h3::Connection> {
    pub fn new(inner: quiche::Connection) -> Self {
        Self {
            inner,
            http3: None,
            partial_responses: PartialRpcResponses::new(),
            partial_requests: PartialRpcRequests::new(),
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
    pub fn handle_response(&mut self, response: RpcResponse) -> Result<(), Error> {
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
                match http3_cx.send_response(&mut self.inner, stream_id, &headers.inner, false) {
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
                    self.partial_responses.remove(&stream_id);
                    return Ok(());
                }
            };

            // Remove the partial response if all the data has been written.
            if response.written == response.body.len() {
                self.partial_responses.remove(&stream_id);
            }
        };

        Ok(())
    }

    /// Handle HTTP/3 Connection Requests.
    pub fn read_http3_requests(
        &mut self,
        req_tx: &RpcRequestSender,
        input_buffer: &mut Vec<u8>,
    ) -> Result<(), Error> {
        loop {
            let http3_cx = self.http3.as_mut().ok_or(Error::Http3ConnectionNotFound)?;

            match http3_cx.poll(&mut self.inner) {
                Ok((stream_id, quiche::h3::Event::Headers { list, has_body })) => {
                    info!("Received HTTP/3 Headers Event: {list:?}");

                    let headers = RpcHeaders::from(list);

                    if has_body {
                        // Event::Data should follow this event.
                        // Store the request headers in the current_request.
                        self.partial_requests
                            .insert(RpcRequest::without_body(stream_id, headers));
                    } else {
                        // If there is no body with this request, handle the request without a body.
                        self.handle_request(RpcRequest::without_body(stream_id, headers), req_tx)?;
                    }
                }

                Ok((stream_id, quiche::h3::Event::Data)) => {
                    info!(
                        "{} received data from stream id {}",
                        self.inner.trace_id(),
                        stream_id
                    );

                    let request = self.partial_requests.get_mut(&stream_id).ok_or_else(|| {
                        error!("Received data for unknown stream id {}", stream_id);
                        Error::UnknownStreamId(stream_id)
                    })?;

                    // Poll recv_body until the body is fully read.
                    let expected_body_size = request.headers.content_length().unwrap_or_default();

                    'recv_body: while expected_body_size > request.body_size() {
                        info!("Current Body Size: {}", request.body_size());

                        match http3_cx.recv_body(&mut self.inner, stream_id, input_buffer) {
                            Ok(received) => {
                                info!("Received {} bytes", received);
                                match request.body.as_mut() {
                                    None => {
                                        request.body = Some(input_buffer[..received].to_vec());
                                    }
                                    Some(body) => {
                                        body.extend_from_slice(&input_buffer[..received]);
                                    }
                                }
                            }
                            Err(quiche::h3::Error::Done) => {
                                info!("Finished reading body from stream {stream_id}");

                                // debug!("Request: {request:?}");

                                // self.handle_request(request.to_owned(), req_tx)?;
                                // current_request = None;

                                break 'recv_body;
                            }
                            Err(e) => {
                                error!(
                                    "Failed to read http3 request body from stream {stream_id}: {:?}",
                                    e
                                );
                                break 'recv_body;
                            }
                        }
                    }

                    debug!("Checking if request is complete...");
                    if request.is_complete() {
                        info!("Handling Complete Request: {}", self.inner.trace_id());
                        let request =
                            self.partial_requests.remove(&stream_id).ok_or_else(|| {
                                error!("Received data for unknown stream id {}", stream_id);
                                Error::UnknownStreamId(stream_id)
                            })?;
                        self.handle_request(request, req_tx)?;
                    }
                }
                // Stream closed.
                Ok((_stream_id, quiche::h3::Event::Finished)) => {}
                // Stream reset.
                Ok((_stream_id, quiche::h3::Event::Reset { .. })) => (),
                Ok((_prioritized_element_id, quiche::h3::Event::PriorityUpdate)) => (),
                Ok((_goaway_id, quiche::h3::Event::GoAway)) => (),
                // No remaining work on the connection.
                Err(quiche::h3::Error::Done) => {
                    info!("{} HTTP/3 Read Requests Done", self.inner.trace_id());
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

    pub fn handle_request(
        &mut self,
        request: RpcRequest,
        req_tx: &RpcRequestSender,
    ) -> Result<(), Error> {
        // Suspend the inner stream while the http3 stream is being processed.
        // This is to avoid the inner stream being polled while the http3 stream is being processed.
        self.suspend_stream(request.stream_id)?;

        // Create a sync channel to send the request to the processor.
        let (res_tx, res_rx) = RpcResponse::oneshot();

        // Send the request to the processor.
        req_tx.send((request, res_tx))?;

        // spawn a task to handle the response.
        match std::thread::spawn(move || res_rx.recv()).join() {
            Ok(Ok(response)) => {
                // Send the response to the http3 stream.
                self.handle_response(response)
            }
            Ok(Err(e)) => {
                error!("Failed to receive response: {:?}", e);
                Err(Error::RecvError(e))
            }
            Err(e) => {
                error!("Failed to join response task: {:?}", e);
                Err(Error::ThreadJoinError(e))
            }
        }
    }

    /// Close the connection
    pub fn close(&mut self, error: TransportErrorCode) -> Result<(), Error> {
        let (inform_peer, code, reason) = error.into_parts();

        self.inner.close(inform_peer, code, reason.as_bytes())?;

        Ok(())
    }
}

pub type RpcConnectionsLock<'a> = std::sync::MutexGuard<
    'a,
    std::collections::HashMap<
        quiche::ConnectionId<'static>,
        RpcConnection<quiche::Connection, quiche::h3::Connection>,
    >,
>;

// pub type RpcConnectionsLockError<'a> = std::sync::PoisonError<
//     std::sync::MutexGuard<
//         'a,
//         std::collections::HashMap<
//             quiche::ConnectionId<'static>,
//             RpcConnection<quiche::Connection, quiche::h3::Connection>,
//         >,
//     >,
// >;

pub struct RpcConnections<Id, Conn, Http3Conn>(Mutex<HashMap<Id, RpcConnection<Conn, Http3Conn>>>);

impl RpcConnections<quiche::ConnectionId<'static>, quiche::Connection, quiche::h3::Connection> {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn lock(&self) -> Result<RpcConnectionsLock, Error> {
        self.0.lock().map_err(|_| Error::ConnectionsLockPoisioned)
    }

    pub fn timeout(&self) -> Result<(), Error> {
        if let Ok(mut lock) = self.lock() {
            lock.iter_mut().for_each(|(_, c)| c.inner.on_timeout());
        }

        Ok(())
    }
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
