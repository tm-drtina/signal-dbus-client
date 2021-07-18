use libsignal_protocol::{DeviceId, ProtocolAddress};

#[derive(Debug, Clone)]
pub(super) struct ProtocolAddressBytes(Box<[u8]>);

impl ProtocolAddressBytes {
	// TODO: use or remove
	#[allow(dead_code)]
    pub(super) fn prefix(&self) -> &[u8] {
        &self.0[..self.0.len() - std::mem::size_of::<DeviceId>()]
    }
}

impl AsRef<[u8]> for ProtocolAddressBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&ProtocolAddress> for ProtocolAddressBytes {
    fn from(addr: &ProtocolAddress) -> Self {
        let str_bytes = addr.name().bytes();
        let device_id_bytes = addr.device_id().to_le_bytes();
        Self(str_bytes.chain(device_id_bytes).collect())
    }
}

pub(super) fn sled_to_signal_error(
    call: &'static str,
    err: sled::Error,
) -> libsignal_protocol::SignalProtocolError {
    libsignal_protocol::error::SignalProtocolError::InvalidState(call, err.to_string())
}
