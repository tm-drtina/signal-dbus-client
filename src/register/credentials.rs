use libsignal_protocol::{IdentityKeyPair, ProtocolAddress};

pub struct Credentials {
    pub aci_identity_key_pair: IdentityKeyPair,
    pub registration_id: u32,
    pub address: ProtocolAddress,
    pub api_pass: String,
}
