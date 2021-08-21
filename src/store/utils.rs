use libsignal_protocol::{DeviceId, ProtocolAddress};

#[derive(Debug, Clone)]
pub(super) struct ProtocolAddressBytes(Box<[u8]>);

const DEVICE_ID_SIZE: usize = std::mem::size_of::<DeviceId>();

impl ProtocolAddressBytes {
    pub(super) fn new(bytes: Box<[u8]>) -> Self {
        Self(bytes)
    }

    pub(super) fn name_bytes(&self) -> &[u8] {
        &self.0[..self.0.len() - DEVICE_ID_SIZE]
    }

    pub(super) fn device_id_bytes(&self) -> [u8; DEVICE_ID_SIZE] {
        let mut buf = [0u8; DEVICE_ID_SIZE];
        buf.copy_from_slice(&self.0[self.0.len() - DEVICE_ID_SIZE..]);
        buf
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

impl From<ProtocolAddressBytes> for ProtocolAddress {
    fn from(bytes: ProtocolAddressBytes) -> Self {
        let name_bytes = bytes.name_bytes();
        let name = String::from_utf8_lossy(name_bytes).to_string();

        let device_id_bytes = bytes.device_id_bytes();
        let device_id = DeviceId::from_le_bytes(device_id_bytes);

        Self::new(name, device_id)
    }
}

pub(super) fn sled_to_signal_error(
    call: &'static str,
    err: sled::Error,
) -> libsignal_protocol::SignalProtocolError {
    libsignal_protocol::error::SignalProtocolError::InvalidState(call, err.to_string())
}
