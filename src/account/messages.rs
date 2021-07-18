use std::time::SystemTime;

use libsignal_protocol::{CiphertextMessage, DeviceId};
use serde::Serialize;

use crate::utils::serde::serialize_ciphertext_message;

#[derive(Serialize)]
pub(crate) struct SendMetadata {
    #[serde(rename = "type")]
    msg_type: u8,
    #[serde(rename = "destinationDeviceId")]
    device_id: DeviceId,
    #[serde(rename = "destinationRegistrationId")]
    registration_id: u32,
    #[serde(serialize_with = "serialize_ciphertext_message")]
    content: CiphertextMessage,
}

impl SendMetadata {
    pub(crate) fn new(
        content: CiphertextMessage,
        device_id: DeviceId,
        registration_id: u32,
    ) -> Self {
        Self {
            msg_type: content.message_type() as u8,
            device_id,
            registration_id,
            content,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct MessagesWrapper {
    destination: String,
    messages: Vec<SendMetadata>,
    timestamp: u64,
    online: bool,
}

impl MessagesWrapper {
    pub(crate) fn new(destination: String, messages: Vec<SendMetadata>) -> Self {
        Self {
            destination,
            messages,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
            online: false,
        }
    }
}
