use libsignal_protocol::IdentityKeyPair;

pub struct Credentials {
    pub identity_key_pair: IdentityKeyPair,
    pub registration_id: u32,
    pub api_user: String,
    pub api_pass: String,
}
