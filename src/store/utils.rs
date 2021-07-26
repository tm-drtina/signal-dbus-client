use libsignal_protocol::{DeviceId, ProtocolAddress};
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub(super) struct ProtocolAddressBytes(Box<[u8]>);

impl ProtocolAddressBytes {
    pub(super) fn prefix(&self) -> &[u8] {
        &self.0[..self.0.len() - std::mem::size_of::<DeviceId>()]
    }
    pub(super) fn device_id(&self) -> DeviceId {
        let bytes = &self.0[self.0.len() - std::mem::size_of::<DeviceId>()..];
        DeviceId::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl AsRef<[u8]> for ProtocolAddressBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for ProtocolAddressBytes {
    fn from(addr: &[u8]) -> Self {
        Self(Box::from(addr))
    }
}
impl From<ProtocolAddressBytes> for ProtocolAddress {
    fn from(bytes: ProtocolAddressBytes) -> Self {
        let name = String::from_utf8(bytes.prefix().to_vec()).expect("Only valid stings are serialized");
        let device_id = bytes.device_id();
        Self::new(name, device_id)
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
