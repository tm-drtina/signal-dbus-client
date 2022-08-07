use std::collections::HashMap;

use base64::STANDARD_NO_PAD;
use hyper::Method;
use libsignal_protocol::ProtocolAddress;
use signal_provisioning_api::ProvisionMessage;

use rand::{rngs::OsRng, RngCore};

use serde::{Deserialize, Serialize};

use super::credentials::Credentials;
use crate::common::{ApiConfig, ApiPath};
use crate::error::Result;
use crate::utils::HttpClient;

#[derive(Serialize, Debug)]
struct DeviceRegistrationRequest {
    capabilities: HashMap<String, bool>,
    #[serde(rename = "fetchesMessages")]
    fetches_messages: bool,
    name: String,
    #[serde(rename = "registrationId")]
    registration_id: u32,
    #[serde(rename = "supportsSms")]
    supports_sms: bool,
    #[serde(rename = "unidentifiedAccessKey")]
    unidentified_access_key: Option<String>,
    #[serde(rename = "unrestrictedUnidentifiedAccess")]
    unrestricted_unidentified_access: bool,
}

impl DeviceRegistrationRequest {
    pub fn new(name: String, registration_id: u32) -> Self {
        // TODO: we send capabilities that we don't have!
        let mut capabilities = HashMap::new();
        capabilities.insert("gv2-3".to_string(), true);
        capabilities.insert("gv1-migration".to_string(), true);
        capabilities.insert("senderKey".to_string(), true);
        capabilities.insert("changeNumber".to_string(), true);
        capabilities.insert("announcementGroup".to_string(), true);
        capabilities.insert("giftBadges".to_string(), true);
        capabilities.insert("stories".to_string(), true);
        Self {
            capabilities,
            fetches_messages: true,
            name,
            registration_id,
            supports_sms: false,
            unidentified_access_key: None,
            unrestricted_unidentified_access: false,
        }
    }
}

#[derive(Deserialize, Debug)]
struct DeviceRegistrationResponse {
    uuid: Option<String>,
    #[serde(rename = "deviceId")]
    device_id: Option<u32>,
}

pub(super) async fn register_device(
    api_config: &ApiConfig,
    message: ProvisionMessage,
    name: &str,
) -> Result<Credentials> {
    let registration_id = (OsRng.next_u32() as u32) & 0x00003fff;
    // Should we encrypt device name as in TS sources?
    let registration_request = DeviceRegistrationRequest::new(name.to_string(), registration_id);

    let mut api_pass = [0u8; 16];
    OsRng.fill_bytes(&mut api_pass);
    let api_pass = base64::encode_config(api_pass, STANDARD_NO_PAD);

    let http_client = HttpClient::new(message.number(), &api_pass, api_config)?;
    let response: DeviceRegistrationResponse = http_client
        .send_json(
            Method::PUT,
            ApiPath::Device {
                provisioning_code: message.provisioning_code(),
            },
            &registration_request,
        )
        .await?
        .json()
        .await?;

    let address = ProtocolAddress::new(
        response
            .uuid
            .unwrap_or_else(|| message.number().to_string()),
        response.device_id.unwrap_or(1).into(),
    );

    Ok(Credentials {
        address,
        api_pass,
        identity_key_pair: *message.identity_key_pair(),
        registration_id,
    })
}
