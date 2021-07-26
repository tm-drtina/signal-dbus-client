use std::time::SystemTime;

use libsignal_protocol::{CiphertextMessage, DeviceId};
use serde::Serialize;

use crate::utils::serde::serialize_ciphertext_message;

#[derive(Serialize)]
pub(crate) struct SendMetadata {
    #[serde(rename = "type")]
    msg_type: u8,
    destination: String,
    #[serde(rename = "destinationDeviceId")]
    device_id: DeviceId,
    #[serde(rename = "destinationRegistrationId")]
    registration_id: u32,
    body: String,
    #[serde(serialize_with = "serialize_ciphertext_message")]
    content: CiphertextMessage,
    relay: String,
    timestamp: u64,
}

impl SendMetadata {
    pub(crate) fn new(
        content: CiphertextMessage,
        destination: String,
        device_id: DeviceId,
        registration_id: u32,
    ) -> Self {
        Self {
            msg_type: content.message_type() as u8,
            destination,
            device_id,
            registration_id,
            body: String::new(),
            content,
            relay: String::new(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct MessagesWrapper {
    messages: Vec<SendMetadata>,
    timestamp: u64,
    online: bool,
}

impl MessagesWrapper {
    pub(crate) fn new(messages: Vec<SendMetadata>) -> Self {
        Self {
            messages,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
            online: false,
        }
    }
}
