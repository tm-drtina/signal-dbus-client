use std::convert::TryFrom;

use base64::engine::{Engine as _, general_purpose::{STANDARD, STANDARD_NO_PAD}};
use libsignal_protocol::{
    CiphertextMessage, DeviceId, IdentityKey, PreKeyId, PublicKey, SignedPreKeyId,
};
use serde::{de::Error, Deserialize, Deserializer, Serializer};

pub(crate) fn deserialize_identity_key<'de, D>(deserializer: D) -> Result<IdentityKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let decoded = STANDARD_NO_PAD.decode(s).expect("Valid base64.");
    IdentityKey::try_from(&decoded[..]).map_err(D::Error::custom)
}

pub(crate) fn serialize_identity_key<S>(
    value: &IdentityKey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = STANDARD_NO_PAD.encode(value.serialize());
    serializer.serialize_str(&encoded)
}

pub(crate) fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let decoded = STANDARD_NO_PAD.decode(s).expect("Valid base64.");
    PublicKey::try_from(&decoded[..]).map_err(D::Error::custom)
}

pub(crate) fn serialize_public_key<S>(value: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = STANDARD_NO_PAD.encode(value.serialize());
    serializer.serialize_str(&encoded)
}

pub(crate) fn deserialize_byte_vec<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    STANDARD_NO_PAD.decode(s).map_err(D::Error::custom)
}

pub(crate) fn serialize_byte_vec<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = STANDARD_NO_PAD.encode(value);
    serializer.serialize_str(&encoded)
}

pub(crate) fn serialize_ciphertext_message<S>(
    value: &CiphertextMessage,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = STANDARD.encode(value.serialize());
    serializer.serialize_str(&encoded)
}

pub(crate) fn deserialize_device_id<'de, D>(deserializer: D) -> Result<DeviceId, D::Error>
where
    D: Deserializer<'de>,
{
    let num: u32 = Deserialize::deserialize(deserializer)?;
    Ok(DeviceId::from(num))
}

pub(crate) fn deserialize_device_id_vec<'de, D>(deserializer: D) -> Result<Vec<DeviceId>, D::Error>
where
    D: Deserializer<'de>,
{
    let nums: Vec<u32> = Deserialize::deserialize(deserializer)?;
    Ok(nums.into_iter().map(DeviceId::from).collect())
}

pub(crate) fn serialize_device_id<S>(value: &DeviceId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32((*value).into())
}

pub(crate) fn deserialize_pre_key_id<'de, D>(deserializer: D) -> Result<PreKeyId, D::Error>
where
    D: Deserializer<'de>,
{
    let num: u32 = Deserialize::deserialize(deserializer)?;
    Ok(PreKeyId::from(num))
}

pub(crate) fn serialize_pre_key_id<S>(value: &PreKeyId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32((*value).into())
}

pub(crate) fn deserialize_signed_pre_key_id<'de, D>(
    deserializer: D,
) -> Result<SignedPreKeyId, D::Error>
where
    D: Deserializer<'de>,
{
    let num: u32 = Deserialize::deserialize(deserializer)?;
    Ok(SignedPreKeyId::from(num))
}

pub(crate) fn serialize_signed_pre_key_id<S>(
    value: &SignedPreKeyId,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32((*value).into())
}
