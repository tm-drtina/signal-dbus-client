use hyper::StatusCode;

#[derive(Debug)]
pub enum Error {
    SignalProtocolError(libsignal_protocol::SignalProtocolError),
    SignalCryptoError(signal_provisioning_api::SignalCryptoError),
    SocketError(tungstenite::Error),
    HttpParserError(tungstenite::http::Error),
    HttpError(StatusCode, String),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    HyperError(hyper::Error),
    SledError(sled::Error),

    ProvisioningFailed,
    ConfigError(String),
    EmptyResponse,
    ConnectionError(String),
    Uninitialized,
}

impl From<signal_provisioning_api::Error> for Error {
    fn from(err: signal_provisioning_api::Error) -> Self {
        match err {
            signal_provisioning_api::Error::SignalCryptoError(err) => Self::SignalCryptoError(err),
            signal_provisioning_api::Error::SignalProtocolError(err) => {
                Self::SignalProtocolError(err)
            }
        }
    }
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::SocketError(err)
    }
}

impl From<tungstenite::http::Error> for Error {
    fn from(err: tungstenite::http::Error) -> Self {
        Self::HttpParserError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err)
    }
}

impl From<libsignal_protocol::SignalProtocolError> for Error {
    fn from(err: libsignal_protocol::SignalProtocolError) -> Self {
        Self::SignalProtocolError(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Self::HyperError(err)
    }
}

impl From<sled::Error> for Error {
    fn from(err: sled::Error) -> Self {
        Self::SledError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}
impl std::error::Error for Error {}
