use signal_provisioning_api::{
    ApiConfig, DeviceRegistrationRequest, DeviceRegistrationResponse, ProvisionMessage,
};

use rand::{rngs::OsRng, RngCore};

use tungstenite::http::{Method, Request, Response};

use crate::error::Result;
use crate::utils::send_json;
use crate::register::Credentials;

pub(super) fn register_device(message: ProvisionMessage, name: &str) -> Result<Credentials> {
    let api_config = ApiConfig::default();
    let registration_id = (OsRng.next_u32() as u16) & 0x3fff;
    // Should we encrypt device name as in TS sources?
    let data = DeviceRegistrationRequest::new(name.to_string(), registration_id);

    let mut password = [0u8; 16];
    OsRng.fill_bytes(&mut password);
    let password = base64::encode(password);
    let password = &password[..password.len() - 2];

    let auth = base64::encode(format!("{}:{}", message.number(), password));
    let api_uri = format!(
        "{}{}{}{}",
        api_config.http_protocol,
        api_config.host,
        api_config.devices_path,
        message.provisioning_code()
    );

    let req = Request::builder()
        .method(Method::PUT)
        .uri(api_uri)
        .header("Authorization", format!("Basic {}", auth))
        .header("User-Agent", "Signal-Desktop/5.7.1")
        .body(data)
        .unwrap();

    let response: Response<DeviceRegistrationResponse> = send_json(&api_config, req)?;
    let response_body = response.into_body();
    let username = format!(
        "{}.{}",
        response_body
            .uuid
            .unwrap_or_else(|| message.number().clone()),
        response_body.deviceId.unwrap_or(1)
    );

    Ok(Credentials {
        username,
        password: password.to_string(),
        identity_key_pair: *message.identity_key_pair(),
    })
}
