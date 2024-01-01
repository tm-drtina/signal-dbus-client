use tokio_tungstenite::tungstenite;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SignalProtocolError(#[from] libsignal_protocol::SignalProtocolError),
    #[error(transparent)]
    SignalCryptoError(#[from] signal_provisioning_api::SignalCryptoError),
    #[error(transparent)]
    ProvisioningApiError(#[from] signal_provisioning_api::Error),
    #[error(transparent)]
    SocketError(#[from] tungstenite::Error),
    #[error(transparent)]
    HttpParserError(#[from] tungstenite::http::Error),
    #[error("http error {0}: {1}")]
    HttpError(reqwest::StatusCode, String),
    #[error("Deprecated http error: {0}")]
    DeprecatedHttpError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SledError(#[from] sled::Error),
    #[error(transparent)]
    UuidParsingError(#[from] uuid::Error),

    #[error("Provisioning failed")]
    ProvisioningFailed,
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    #[error("App state is not initialized!")]
    Uninitialized,
}

pub type Result<T> = std::result::Result<T, Error>;
