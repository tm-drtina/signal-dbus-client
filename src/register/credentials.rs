use std::convert::TryFrom;

use libsignal_protocol::IdentityKeyPair;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    #[serde(
        deserialize_with = "parse_identity_key_pair",
        serialize_with = "serialize_identity_key_pair"
    )]
    pub identity_key_pair: IdentityKeyPair,
}

fn parse_identity_key_pair<'de, D>(deserializer: D) -> Result<IdentityKeyPair, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let decoded = base64::decode(s).expect("Valid base64.");
    IdentityKeyPair::try_from(&decoded[..]).map_err(D::Error::custom)
}

fn serialize_identity_key_pair<S>(value: &IdentityKeyPair, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = base64::encode(value.serialize());
    serializer.serialize_str(&encoded)
}
