use std::str::FromStr;

use http::uri::{Authority, PathAndQuery};
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

use crate::error::{Error, Result};

pub struct ApiConfig {
    pub user_agent: String,
    pub authority: Authority,
    pub cert_bytes: Box<[u8]>,
}

impl ApiConfig {
    pub(crate) fn rustls_config(&self) -> Result<ClientConfig> {
        let certs = rustls_pemfile::certs(&mut &*self.cert_bytes).collect::<std::result::Result<Vec<_>, _>>().map_err(|_| {
            Error::ConfigError(String::from("Failed to process Signal certificate"))
        })?;

        let mut root_store = RootCertStore::empty();
        root_store.add_parsable_certificates(certs);

        Ok(ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth())
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        let cert_bytes = &include_bytes!("./signal_certs.pem")[..];

        Self {
            user_agent: "Signal-Desktop/6.10.1 Linux".to_string(),
            authority: Authority::from_static("textsecure-service.whispersystems.org:443"),
            cert_bytes: Vec::from(cert_bytes).into_boxed_slice(),
        }
    }
}

pub enum ApiPath<'a> {
    ProvisioningSocket,
    Device {
        provisioning_code: &'a str,
    },
    PreKeys,
    SendMessage {
        recipient: &'a str,
    },
    GetSessionKey {
        recipient: &'a str,
        device_id: &'a str,
    },
}

impl<'a> ApiPath<'a> {
    pub fn get_path(self) -> PathAndQuery {
        match self {
            Self::ProvisioningSocket => PathAndQuery::from_static("/v1/websocket/provisioning/"),
            Self::Device { provisioning_code } => {
                PathAndQuery::from_str(&format!("/v1/devices/{}", provisioning_code)).unwrap()
            }
            Self::PreKeys => PathAndQuery::from_static("/v2/keys/"),
            Self::SendMessage { recipient } => {
                PathAndQuery::from_str(&format!("/v1/messages/{}", recipient)).unwrap()
            }
            Self::GetSessionKey {
                recipient,
                device_id,
            } => PathAndQuery::from_str(&format!("/v2/keys/{}/{}", recipient, device_id)).unwrap(),
        }
    }
}
