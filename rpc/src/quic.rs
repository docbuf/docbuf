use std::path::PathBuf;

use crate::error::Error;

use tracing::debug;

/// Max Quic Datagram Size
pub const MAX_QUIC_DATAGRAM_SIZE: usize = 1350;

// Default (Development) Quic Config Setting Constants
pub const DEFAULT_MAX_IDLE_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_INITIAL_MAX_DATA: u64 = 10_000_000;
pub const DEFAULT_INITIAL_MAX_STREAM_DATA_BIDI_LOCAL: u64 = 1_000_000;
pub const DEFAULT_INITIAL_MAX_STREAM_DATA_BIDI_REMOTE: u64 = 1_000_000;
pub const DEFAULT_INITIAL_MAX_STREAM_DATA_UNI: u64 = 1_000_000;
pub const DEFAULT_INITIAL_MAX_STREAMS_BIDI: u64 = 100;
pub const DEFAULT_INITIAL_MAX_STREAMS_UNI: u64 = 100;

// CryptoError Code Range for Quic Transport Crypto Errors
pub const CRYPTO_ERROR_RANGE: std::ops::Range<u16> = 0x0100..0x01ff;

/// Wrapper around Quic implementation config struct, e.g., `quiche::Config`.
#[derive(Clone)]
pub struct QuicConfig<T: Clone>(pub T);

impl Into<quiche::Config> for QuicConfig<quiche::Config> {
    fn into(self) -> quiche::Config {
        self.0
    }
}

impl AsMut<quiche::Config> for QuicConfig<quiche::Config> {
    fn as_mut(&mut self) -> &mut quiche::Config {
        &mut self.0
    }
}

/// Default QuicConfig implementation wrapping a quiche::Config.
impl QuicConfig<quiche::Config> {
    /// Create a new Quic Config using the Quiche Config struct.
    pub fn new() -> Result<Self, Error> {
        Ok(Self(quiche::Config::new(quiche::PROTOCOL_VERSION)?))
    }

    pub fn default() -> Result<Self, Error> {
        let mut config = Self::new()?;
        config.set_default()?;
        Ok(config)
    }

    /// The default (DEVELOPER) Quic Quiche Config settings.
    ///
    /// WARNING: DO NOT USE THIS IN PRODUCTION. For production,
    /// use `QuicConfig::production()` or custom settings.
    fn set_default(&mut self) -> Result<(), Error> {
        let config = self.as_mut();

        // Ignore verifying the peer's certificate.
        // WARNING: This is insecure and should not be used in production.
        config.verify_peer(false);

        // Set application protocol to HTTP/3.
        config.set_application_protos(quiche::h3::APPLICATION_PROTOCOL)?;

        // Set the max idle timeout
        config.set_max_idle_timeout(DEFAULT_MAX_IDLE_TIMEOUT_MS);

        // Set the max datagram receive and send payload sizes
        config.set_max_recv_udp_payload_size(MAX_QUIC_DATAGRAM_SIZE);
        config.set_max_send_udp_payload_size(MAX_QUIC_DATAGRAM_SIZE);
        config.set_initial_max_data(DEFAULT_INITIAL_MAX_DATA);

        // Set initial max stream data sizes
        config.set_initial_max_stream_data_bidi_local(DEFAULT_INITIAL_MAX_STREAM_DATA_BIDI_LOCAL);
        config.set_initial_max_stream_data_bidi_remote(DEFAULT_INITIAL_MAX_STREAM_DATA_BIDI_REMOTE);
        config.set_initial_max_stream_data_uni(DEFAULT_INITIAL_MAX_STREAM_DATA_UNI);
        config.set_initial_max_streams_bidi(DEFAULT_INITIAL_MAX_STREAMS_BIDI);
        config.set_initial_max_streams_uni(DEFAULT_INITIAL_MAX_STREAMS_UNI);

        config.set_disable_active_migration(true);
        config.enable_early_data();

        // Set the ACK delay to 0.
        config.set_ack_delay_exponent(0);
        config.enable_pacing(false);

        Ok(())
    }

    /// Developer-friendly configurations for Quic.
    /// DO NOT USE THIS IN PRODUCTION,
    /// use `QuicConfig::production()` or custom settings instead.
    pub fn development(tls_options: TlsOptions) -> Result<Self, Error> {
        let mut config = Self::from_tls_options(tls_options)?;

        config.set_default()?;

        Ok(config)
    }

    /// From TLS Options, create a Quic Config for a client.
    pub fn from_tls_options(tls_options: TlsOptions) -> Result<Self, Error> {
        let cert_chain = tls_options.cert();
        let key = tls_options.key();

        let mut config = Self(match tls_options {
            TlsOptions::Client { .. } => quiche::Config::new(quiche::PROTOCOL_VERSION)?,
            TlsOptions::Server {
                boring_ctx_builder, ..
            } => match boring_ctx_builder {
                Some(tls_ctx_builder) => quiche::Config::with_boring_ssl_ctx_builder(
                    quiche::PROTOCOL_VERSION,
                    tls_ctx_builder,
                )?,
                None => quiche::Config::new(quiche::PROTOCOL_VERSION)?,
            },
            TlsOptions::None => {
                let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
                config.verify_peer(false);
                config
            }
        });

        if let (Some(cert), Some(key)) = (cert_chain, key) {
            config.set_tls_ctx(cert, key)?;
        }

        Ok(config)
    }

    /// The `production` Quick Quiche Config settings.
    pub fn production(tls_options: TlsOptions) -> Result<Self, Error> {
        unimplemented!("QuicConfig::production")
    }

    /// Set the TLS context for the Quic Config.
    pub fn set_tls_ctx(&mut self, cert_chain: PathBuf, key: PathBuf) -> Result<(), Error> {
        let config = self.as_mut();

        match (cert_chain.to_str(), key.to_str()) {
            (Some(cert_chain), Some(key)) => {
                // let cert_chain_bytes = std::fs::read(cert_chain)?;
                let key_bytes = std::fs::read(key)?;
                let pkey = boring::pkey::PKey::private_key_from_pem(&key_bytes)
                    .expect("failed to load private key");

                debug!("Loaded private key from file: {:?}", pkey);

                config.load_cert_chain_from_pem_file(cert_chain)?;
                config.load_priv_key_from_pem_file(key)?;
            }
            _ => {
                return Err(Error::TlsError(
                    "Failed to load certificate chain and key".to_string(),
                ));
            }
        };

        Ok(())
    }
}

/// Quic Transport Error Codes
/// See: https://www.rfc-editor.org/rfc/rfc9000#section-20.1
#[derive(Debug, thiserror::Error)]
pub enum TransportErrorCode {
    NoError,
    InternalError,
    ConnectionRefused,
    FlowControlError,
    StreamLimitError,
    StreamStateError,
    FinalSizeError,
    FrameEncodingError,
    TransportParameterError,
    ConnectionIdLimitError,
    ProtocolViolation,
    InvalidToken,
    ApplicationError(String),
    CryptoBufferExceeded,
    KeyUpdateError,
    AeadLimitReached,
    NoViablePath,
    CryptoError(u16),
    Unknown(u16),
}

impl TransportErrorCode {
    pub fn reason(&self) -> String {
        match self {
            TransportErrorCode::NoError => format!("No Error"),
            TransportErrorCode::InternalError => format!("Internal Error"),
            TransportErrorCode::ConnectionRefused => format!("Connection Refused"),
            TransportErrorCode::FlowControlError => format!("Flow Control Error"),
            TransportErrorCode::StreamLimitError => format!("Stream Limit Error"),
            TransportErrorCode::StreamStateError => format!("Stream State Error"),
            TransportErrorCode::FinalSizeError => format!("Final Size Error"),
            TransportErrorCode::FrameEncodingError => format!("Frame Encoding Error"),
            TransportErrorCode::TransportParameterError => format!("Transport Parameter Error"),
            TransportErrorCode::ConnectionIdLimitError => format!("Connection ID Limit Error"),
            TransportErrorCode::ProtocolViolation => format!("Protocol Violation"),
            TransportErrorCode::InvalidToken => format!("Invalid Token"),
            TransportErrorCode::ApplicationError(reason) => format!("Application Error: {reason}"),
            TransportErrorCode::CryptoBufferExceeded => format!("Crypto Buffer Exceeded"),
            TransportErrorCode::KeyUpdateError => format!("Key Update Error"),
            TransportErrorCode::AeadLimitReached => format!("AEAD Limit Reached"),
            TransportErrorCode::NoViablePath => format!("No Viable Path"),
            TransportErrorCode::CryptoError(code) => format!("Crypto Error: {code}"),
            TransportErrorCode::Unknown(code) => format!("Unknown Error: {code}"),
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            TransportErrorCode::NoError => 0x0,
            TransportErrorCode::InternalError => 0x1,
            TransportErrorCode::ConnectionRefused => 0x2,
            TransportErrorCode::FlowControlError => 0x3,
            TransportErrorCode::StreamLimitError => 0x4,
            TransportErrorCode::StreamStateError => 0x5,
            TransportErrorCode::FinalSizeError => 0x6,
            TransportErrorCode::FrameEncodingError => 0x7,
            TransportErrorCode::TransportParameterError => 0x8,
            TransportErrorCode::ConnectionIdLimitError => 0x9,
            TransportErrorCode::ProtocolViolation => 0xa,
            TransportErrorCode::InvalidToken => 0xb,
            TransportErrorCode::ApplicationError(_) => 0xc,
            TransportErrorCode::CryptoBufferExceeded => 0xd,
            TransportErrorCode::KeyUpdateError => 0xe,
            TransportErrorCode::AeadLimitReached => 0xf,
            TransportErrorCode::NoViablePath => 0x10,
            TransportErrorCode::CryptoError(code) => *code,
            TransportErrorCode::Unknown(code) => *code,
        }
    }

    /// Inform the peer about the error.
    pub fn inform_peer(&self) -> bool {
        match self {
            TransportErrorCode::NoError => false,
            TransportErrorCode::InternalError => false,
            TransportErrorCode::ConnectionRefused => false,
            TransportErrorCode::FlowControlError => false,
            TransportErrorCode::StreamLimitError => false,
            TransportErrorCode::StreamStateError => false,
            TransportErrorCode::FinalSizeError => false,
            TransportErrorCode::FrameEncodingError => false,
            TransportErrorCode::TransportParameterError => false,
            TransportErrorCode::ConnectionIdLimitError => false,
            TransportErrorCode::ProtocolViolation => false,
            TransportErrorCode::InvalidToken => false,
            TransportErrorCode::ApplicationError(_) => true,
            TransportErrorCode::CryptoBufferExceeded => true,
            TransportErrorCode::KeyUpdateError => true,
            TransportErrorCode::AeadLimitReached => true,
            TransportErrorCode::NoViablePath => true,
            TransportErrorCode::CryptoError(_) => true,
            TransportErrorCode::Unknown(_) => true,
        }
    }

    pub fn into_parts(&self) -> (bool, u64, String) {
        (self.inform_peer(), self.code() as u64, self.reason())
    }
}

impl From<u16> for TransportErrorCode {
    fn from(code: u16) -> Self {
        match code {
            0x0 => TransportErrorCode::NoError,
            0x1 => TransportErrorCode::InternalError,
            0x2 => TransportErrorCode::ConnectionRefused,
            0x3 => TransportErrorCode::FlowControlError,
            0x4 => TransportErrorCode::StreamLimitError,
            0x5 => TransportErrorCode::StreamStateError,
            0x6 => TransportErrorCode::FinalSizeError,
            0x7 => TransportErrorCode::FrameEncodingError,
            0x8 => TransportErrorCode::TransportParameterError,
            0x9 => TransportErrorCode::ConnectionIdLimitError,
            0xa => TransportErrorCode::ProtocolViolation,
            0xb => TransportErrorCode::InvalidToken,
            0xc => TransportErrorCode::ApplicationError(Default::default()),
            0xd => TransportErrorCode::CryptoBufferExceeded,
            0xe => TransportErrorCode::KeyUpdateError,
            0xf => TransportErrorCode::AeadLimitReached,
            0x10 => TransportErrorCode::NoViablePath,
            code if CRYPTO_ERROR_RANGE.contains(&code) => TransportErrorCode::CryptoError(code),
            _ => TransportErrorCode::Unknown(code),
        }
    }
}

impl From<TransportErrorCode> for u16 {
    fn from(error: TransportErrorCode) -> Self {
        error.code()
    }
}

impl Into<u64> for TransportErrorCode {
    fn into(self) -> u64 {
        u16::from(self) as u64
    }
}

impl std::fmt::Display for TransportErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (0x{:x})", self.reason(), self.code())
    }
}

/// Quic TLS Context Options, passed to the QuicConfig.
pub enum TlsOptions {
    Client {
        cert_chain: PathBuf,
        key: PathBuf,
    },
    Server {
        cert_chain: PathBuf,
        key: PathBuf,
        /// Optionally, use BoringSSL Context.
        boring_ctx_builder: Option<boring::ssl::SslContextBuilder>,
    },
    None,
}

impl TlsOptions {
    pub fn cert(&self) -> Option<PathBuf> {
        match self {
            TlsOptions::Client { cert_chain, .. } => Some(cert_chain.to_owned()),
            TlsOptions::Server { cert_chain, .. } => Some(cert_chain.to_owned()),
            TlsOptions::None => None,
        }
    }

    pub fn key(&self) -> Option<PathBuf> {
        match self {
            TlsOptions::Client { key, .. } => Some(key.to_owned()),
            TlsOptions::Server { key, .. } => Some(key.to_owned()),
            TlsOptions::None => None,
        }
    }
}
