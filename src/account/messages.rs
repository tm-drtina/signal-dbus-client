use std::time::SystemTime;

use libsignal_protocol::{CiphertextMessage, DeviceId};
use serde::{Deserialize, Serialize};

use crate::utils::serde::{
    deserialize_device_id_vec, serialize_ciphertext_message, serialize_device_id,
};

#[derive(Serialize)]
pub(crate) struct SendMetadata {
    #[serde(rename = "type")]
    msg_type: u8,
    #[serde(rename = "destinationDeviceId", serialize_with = "serialize_device_id")]
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

#[derive(Debug, Deserialize)]
pub(crate) struct MessageResponse409 {
    #[serde(
        rename = "missingDevices",
        deserialize_with = "deserialize_device_id_vec"
    )]
    pub(crate) missing_devices: Vec<DeviceId>,
    #[serde(
        rename = "extraDevices",
        deserialize_with = "deserialize_device_id_vec"
    )]
    pub(crate) extra_devices: Vec<DeviceId>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MessageResponse200 {
    #[serde(rename = "needsSync")]
    #[allow(dead_code)]
    pub(crate) needs_sync: bool,
}
