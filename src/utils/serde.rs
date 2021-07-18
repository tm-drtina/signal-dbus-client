use std::convert::TryFrom;

use base64::STANDARD_NO_PAD;
use libsignal_protocol::{CiphertextMessage, IdentityKey, PublicKey};
use serde::{de::Error, Deserialize, Deserializer, Serializer};

pub(crate) fn deserialize_identity_key<'de, D>(deserializer: D) -> Result<IdentityKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let decoded = base64::decode_config(s, STANDARD_NO_PAD).expect("Valid base64.");
    IdentityKey::try_from(&decoded[..]).map_err(D::Error::custom)
}

pub(crate) fn serialize_identity_key<S>(
    value: &IdentityKey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = base64::encode_config(value.serialize(), STANDARD_NO_PAD);
    serializer.serialize_str(&encoded)
}

pub(crate) fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let decoded = base64::decode_config(s, STANDARD_NO_PAD).expect("Valid base64.");
    PublicKey::try_from(&decoded[..]).map_err(D::Error::custom)
}

pub(crate) fn serialize_public_key<S>(value: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = base64::encode_config(value.serialize(), STANDARD_NO_PAD);
    serializer.serialize_str(&encoded)
}

pub(crate) fn deserialize_byte_vec<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    base64::decode_config(s, STANDARD_NO_PAD).map_err(D::Error::custom)
}

pub(crate) fn serialize_byte_vec<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = base64::encode_config(value, STANDARD_NO_PAD);
    serializer.serialize_str(&encoded)
}

pub(crate) fn serialize_ciphertext_message<S>(value: &CiphertextMessage, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = base64::encode(value.serialize());
    serializer.serialize_str(&encoded)
}
