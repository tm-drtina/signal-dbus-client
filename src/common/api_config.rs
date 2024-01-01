use rustls::{ClientConfig, RootCertStore};

use crate::error::{Error, Result};

pub struct ApiConfig {
    pub user_agent: String,
    pub authority: String,
    pub cert_bytes: Box<[u8]>,
}

impl ApiConfig {
    pub(crate) fn rustls_config(&self) -> Result<ClientConfig> {
        let certs = rustls_pemfile::certs(&mut &*self.cert_bytes)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| {
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
            authority: "textsecure-service.whispersystems.org:443".to_string(),
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
    pub fn get_path(self) -> String {
        match self {
            Self::ProvisioningSocket => "/v1/websocket/provisioning/".to_string(),
            Self::Device { provisioning_code } => {
                format!("/v1/devices/{}", provisioning_code)
            }
            Self::PreKeys => "/v2/keys/".to_string(),
            Self::SendMessage { recipient } => {
                format!("/v1/messages/{}", recipient)
            }
            Self::GetSessionKey {
                recipient,
                device_id,
            } => format!("/v2/keys/{}/{}", recipient, device_id),
        }
    }
}
