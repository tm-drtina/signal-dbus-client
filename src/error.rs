use tungstenite::http::Response;

#[derive(Debug)]
pub enum Error {
    ApiError(signal_provisioning_api::Error),
    SocketError(tungstenite::Error),
    ProvisioningFailed,
    HttpError(Response<String>),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl From<signal_provisioning_api::Error> for Error {
    fn from(err: signal_provisioning_api::Error) -> Self {
        Self::ApiError(err)
    }
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::SocketError(err)
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

pub type Result<T> = std::result::Result<T, Error>;
