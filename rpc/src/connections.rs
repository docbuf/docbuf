use crate::{
    Error, RpcHeaders, RpcPartialResponses, RpcRequest, RpcRequestSender, RpcResponse,
    TransportErrorCode,
};

use std::collections::HashMap;
use std::sync::Mutex;

use tracing::{error, info};

pub struct RpcConnection<Conn, Http3Conn, Header> {
    pub(crate) inner: Conn,
    pub(crate) http3: Option<Http3Conn>,
    pub(crate) partial_responses: RpcPartialResponses<Header>,
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
    pub fn read_http3_requests(
        &mut self,
        req_tx: &RpcRequestSender<quiche::h3::Header>,
    ) -> Result<(), Error> {
        // Keep track of the current request when parsing the body following the headers.
        let mut current_request = None;

        loop {
            let http3_cx = self.http3.as_mut().ok_or(Error::Http3ConnectionNotFound)?;

            match http3_cx.poll(&mut self.inner) {
                Ok((stream_id, quiche::h3::Event::Headers { list, has_body })) => {
                    let headers = RpcHeaders::from(list);
                    if has_body {
                        // Event::Data should follow this event.
                        // Store the request headers in the current_request.
                        current_request = Some(RpcRequest::without_body(stream_id, headers));
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
                            self.handle_request(request.to_owned(), req_tx)?;
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

    pub fn handle_request(
        &mut self,
        request: RpcRequest<quiche::h3::Header>,
        req_tx: &RpcRequestSender<quiche::h3::Header>,
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
