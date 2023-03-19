use libsignal_protocol::{CiphertextMessage, DeviceId};
use serde::{Deserialize, Serialize};

use crate::proto::signal_service;
use crate::utils::current_timestamp;
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
            timestamp: current_timestamp(),
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

pub(super) fn create_data_message(text: String) -> signal_service::DataMessage {
    signal_service::DataMessage {
        body: Some(text),
        attachments: Vec::new(),
        group: None,
        group_v2: None,
        flags: None,
        expire_timer: None,
        profile_key: None,
        timestamp: Some(current_timestamp()),
        quote: None,
        contact: Vec::new(),
        preview: Vec::new(),
        sticker: None,
        required_protocol_version: None,
        is_view_once: None,
        reaction: None,
        delete: None,
        body_ranges: Vec::new(),
        group_call_update: None,
    }
}

pub(super) fn create_message(data_message: signal_service::DataMessage) -> signal_service::Content {
    signal_service::Content {
        data_message: Some(data_message),
        sync_message: None,
        calling_message: None,
        null_message: None,
        receipt_message: None,
        typing_message: None,
        sender_key_distribution_message: None,
        decryption_error_message: None,
    }
}

pub(super) fn create_sync_message(destination_uuid: String, data_message: signal_service::DataMessage) -> signal_service::Content {
    let sent = Some(signal_service::sync_message::Sent {
        destination: None,
        destination_uuid: Some(destination_uuid),
        timestamp: data_message.timestamp,
        message: Some(data_message),
        expiration_start_timestamp: None,
        unidentified_status: Vec::new(),
        is_recipient_update: None,
    });
    let sync_message = Some(signal_service::SyncMessage {
        sent,
        contacts: None,
        groups: None,
        request: None,
        read: Vec::new(),
        blocked: None,
        verified: None,
        configuration: None,
        padding: None,
        sticker_pack_operation: Vec::new(),
        view_once_open: None,
        fetch_latest: None,
        keys: None,
        message_request_response: None,
    });
    signal_service::Content {
        data_message: None,
        sync_message,
        calling_message: None,
        null_message: None,
        receipt_message: None,
        typing_message: None,
        sender_key_distribution_message: None,
        decryption_error_message: None,
    }
}
