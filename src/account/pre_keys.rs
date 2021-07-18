use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

use libsignal_protocol::{
    DeviceId, IdentityKey, IdentityKeyPair, KeyPair, PreKeyBundle, PreKeyRecord, PublicKey,
    SignedPreKeyRecord,
};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::utils::serde::{
    deserialize_byte_vec, deserialize_identity_key, deserialize_public_key, serialize_byte_vec,
    serialize_identity_key, serialize_public_key,
};

#[derive(Debug, Serialize)]
pub(crate) struct PreKeyState {
    #[serde(rename = "identityKey", serialize_with = "serialize_identity_key")]
    identity_key: IdentityKey,
    #[serde(rename = "preKeys")]
    pre_keys: Vec<PreKeyEntity>,
    #[serde(rename = "signedPreKey")]
    signed_pre_key: SignedPreKeyEntity,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PreKeyEntity {
    #[serde(rename = "keyId")]
    key_id: u32,
    #[serde(
        rename = "publicKey",
        serialize_with = "serialize_public_key",
        deserialize_with = "deserialize_public_key"
    )]
    public_key: PublicKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SignedPreKeyEntity {
    #[serde(flatten)]
    pre_key_entity: PreKeyEntity,
    #[serde(
        rename = "signature",
        serialize_with = "serialize_byte_vec",
        deserialize_with = "deserialize_byte_vec"
    )]
    signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DevicePreKeys {
    #[serde(rename = "deviceId")]
    device_id: DeviceId,
    #[serde(rename = "registrationId")]
    registration_id: u32,
    #[serde(rename = "signedPreKey")]
    signed_pre_key: SignedPreKeyEntity,
    #[serde(rename = "preKey")]
    pre_key: PreKeyEntity,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceKeys {
    #[serde(rename = "identityKey", deserialize_with = "deserialize_identity_key")]
    identity_key: IdentityKey,
    devices: Vec<DevicePreKeys>,
}

pub(crate) fn generate_pre_keys_from_id<R: Rng + CryptoRng>(
    n: u32,
    start_index: u32,
    csprng: &mut R,
) -> Vec<PreKeyRecord> {
    (start_index..start_index + n)
        .map(|id| PreKeyRecord::new(id, &KeyPair::generate(csprng)))
        .collect()
}

pub(crate) fn generate_pre_keys<R: Rng + CryptoRng>(n: u32, csprng: &mut R) -> Vec<PreKeyRecord> {
    generate_pre_keys_from_id(n, 1, csprng)
}

pub(crate) fn generate_signed_pre_key<R: Rng + CryptoRng>(
    identity_key_pair: &IdentityKeyPair,
    pre_key_id: u32,
    csprng: &mut R,
) -> Result<SignedPreKeyRecord> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let key = KeyPair::generate(csprng);
    let signature = identity_key_pair
        .private_key()
        .calculate_signature(&*key.public_key.serialize(), csprng)?;
    Ok(SignedPreKeyRecord::new(
        pre_key_id,
        timestamp,
        &key,
        &*signature,
    ))
}

impl PreKeyState {
    pub(crate) fn new(
        identity_key: IdentityKey,
        pre_keys: &[PreKeyRecord],
        signed_pre_key: &SignedPreKeyRecord,
    ) -> Result<Self> {
        let pre_keys = pre_keys
            .iter()
            .map(|pre_key| PreKeyEntity::try_from(pre_key).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;
        let signed_pre_key = SignedPreKeyEntity::new(signed_pre_key)?;

        Ok(Self {
            identity_key,
            pre_keys,
            signed_pre_key,
        })
    }
}

impl PreKeyEntity {
    fn new(key_id: u32, public_key: PublicKey) -> Self {
        Self { key_id, public_key }
    }
}

impl TryFrom<&PreKeyRecord> for PreKeyEntity {
    type Error = Error;
    fn try_from(pre_key: &PreKeyRecord) -> Result<Self> {
        Ok(Self::new(pre_key.id()?, pre_key.public_key()?))
    }
}

impl TryFrom<&SignedPreKeyRecord> for PreKeyEntity {
    type Error = Error;
    fn try_from(signed_pre_key: &SignedPreKeyRecord) -> Result<Self> {
        Ok(Self::new(
            signed_pre_key.id()?,
            signed_pre_key.public_key()?,
        ))
    }
}

impl SignedPreKeyEntity {
    fn new(signed_pre_key: &SignedPreKeyRecord) -> Result<Self> {
        Ok(Self {
            pre_key_entity: PreKeyEntity::try_from(signed_pre_key)?,
            signature: signed_pre_key.signature()?,
        })
    }
}

impl DevicePreKeys {
    fn into_bundle(self, identity_key: IdentityKey) -> Result<PreKeyBundle> {
        PreKeyBundle::new(
            self.registration_id,
            self.device_id,
            Some((self.pre_key.key_id, self.pre_key.public_key)),
            self.signed_pre_key.pre_key_entity.key_id,
            self.signed_pre_key.pre_key_entity.public_key,
            self.signed_pre_key.signature,
            identity_key,
        )
        .map_err(Into::into)
    }
}

impl TryFrom<DeviceKeys> for Vec<PreKeyBundle> {
    type Error = Error;
    fn try_from(device_keys: DeviceKeys) -> Result<Self> {
        let identity_key = device_keys.identity_key;
        device_keys
            .devices
            .into_iter()
            .map(|device| device.into_bundle(identity_key))
            .collect()
    }
}
