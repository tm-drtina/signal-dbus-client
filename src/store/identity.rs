use std::convert::{TryFrom, TryInto};

use async_trait::async_trait;
use libsignal_protocol::error::Result as SignalResult;
use libsignal_protocol::{
    Context, Direction, IdentityKey, IdentityKeyPair, IdentityKeyStore, ProtocolAddress,
    SignalProtocolError,
};
use sled::{Db, Tree};

use crate::error::{Error, Result};

use super::utils::{sled_to_signal_error, ProtocolAddressBytes};

const IDENTITY_KEY_PAIR_KEY: &[u8] = b"identity_key_pair";
const REGISTRATION_ID_KEY: &[u8] = b"registration_id";
const API_USER_KEY: &[u8] = b"api_user";
const API_PASS_KEY: &[u8] = b"api_pass";

pub(crate) struct SledIdentityStore {
    known_keys: Tree,
    credentials: Tree,
}

impl TryFrom<&Db> for SledIdentityStore {
    type Error = sled::Error;
    fn try_from(value: &Db) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            known_keys: value.open_tree("identities")?,
            credentials: value.open_tree("credentials")?,
        })
    }
}

impl SledIdentityStore {
    pub(crate) fn get_api_user(&self) -> Result<String> {
        match self.credentials.get(API_USER_KEY)? {
            Some(value) => Ok(std::str::from_utf8(&value).expect("Only valid strings are stored.").to_string()),
            None => Err(Error::Uninitialized),
        }
    }
    pub(crate) fn get_api_pass(&self) -> Result<String> {
        match self.credentials.get(API_PASS_KEY)? {
            Some(value) => Ok(std::str::from_utf8(&value).expect("Only valid strings are stored.").to_string()),
            None => Err(Error::Uninitialized),
        }
    }
    pub(crate) fn register_new_account(&self, identity_key_pair: IdentityKeyPair, registration_id: u32, api_user: String, api_pass: String) -> Result<()> {
        self.credentials.insert(IDENTITY_KEY_PAIR_KEY, identity_key_pair.serialize())?;
        self.credentials.insert(REGISTRATION_ID_KEY, &registration_id.to_le_bytes())?;
        self.credentials.insert(API_USER_KEY, api_user.as_bytes())?;
        self.credentials.insert(API_PASS_KEY, api_pass.as_bytes())?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl IdentityKeyStore for SledIdentityStore {
    async fn get_identity_key_pair(&self, _ctx: Context) -> SignalResult<IdentityKeyPair> {
        match self.credentials.get(IDENTITY_KEY_PAIR_KEY) {
            Ok(Some(bytes)) => IdentityKeyPair::try_from(&*bytes),
            Ok(None) => Err(SignalProtocolError::InternalError("uninitialized")),
            Err(err) => Err(sled_to_signal_error("get_identity_key_pair", err)),
        }
    }

    async fn get_local_registration_id(&self, _ctx: Context) -> SignalResult<u32> {
        match self.credentials.get(REGISTRATION_ID_KEY) {
            Ok(Some(bytes)) => Ok(u32::from_le_bytes(
                bytes
                    .as_ref()
                    .try_into()
                    .expect("Stored bytes are valid u32"),
            )),
            Ok(None) => Err(SignalProtocolError::InternalError("uninitialized")),
            Err(err) => Err(sled_to_signal_error("get_identity_key_pair", err)),
        }
    }

    async fn is_trusted_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        _direction: Direction,
        _ctx: Context,
    ) -> SignalResult<bool> {
        let key = ProtocolAddressBytes::from(address);
        match self.known_keys.get(key) {
            Ok(None) => {
                Ok(true) // first use
            }
            Ok(Some(bytes)) => Ok(IdentityKey::try_from(&*bytes)? == *identity),
            Err(err) => Err(sled_to_signal_error("is_trusted_identity", err)),
        }
    }

    async fn get_identity(
        &self,
        address: &ProtocolAddress,
        _ctx: Context,
    ) -> SignalResult<Option<IdentityKey>> {
        let key = ProtocolAddressBytes::from(address);
        match self.known_keys.get(key) {
            Ok(None) => Ok(None),
            Ok(Some(bytes)) => Ok(Some(IdentityKey::try_from(&*bytes)?)),
            Err(err) => Err(sled_to_signal_error("get_identity", err)),
        }
    }

    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        _ctx: Context,
    ) -> SignalResult<bool> {
        let key = ProtocolAddressBytes::from(address);
        let current = self
            .known_keys
            .get(key.clone())
            .map_err(|err| sled_to_signal_error("save_identity", err))?;
        let new = identity.serialize();
        match current {
            None => {
                self.known_keys
                    .insert(key, new)
                    .map_err(|err| sled_to_signal_error("save_identity", err))?;
                Ok(false) // new key
            }
            Some(bytes) if bytes == new => {
                Ok(false) // same key
            }
            Some(_) => {
                self.known_keys
                    .insert(key, new)
                    .map_err(|err| sled_to_signal_error("save_identity", err))?;
                Ok(true) // overwrite
            }
        }
    }
}
