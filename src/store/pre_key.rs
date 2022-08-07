use std::convert::TryFrom;

use async_trait::async_trait;
use libsignal_protocol::error::Result as SignalResult;
use libsignal_protocol::{Context, PreKeyId, PreKeyRecord, PreKeyStore, SignalProtocolError};
use sled::{Db, Tree};

use super::utils::sled_to_signal_error;

pub(crate) struct SledPreKeyStore(Tree);

impl TryFrom<&Db> for SledPreKeyStore {
    type Error = sled::Error;
    fn try_from(db: &Db) -> Result<Self, Self::Error> {
        Ok(Self(db.open_tree("pre-keys")?))
    }
}

#[async_trait(?Send)]
impl PreKeyStore for SledPreKeyStore {
    async fn get_pre_key(&self, prekey_id: PreKeyId, _ctx: Context) -> SignalResult<PreKeyRecord> {
        let key = u32::from(prekey_id).to_le_bytes();
        match self.0.get(key) {
            Ok(Some(bytes)) => PreKeyRecord::deserialize(&bytes),
            Ok(None) => Err(SignalProtocolError::InvalidPreKeyId),
            Err(err) => Err(sled_to_signal_error("get_pre_key", err)),
        }
    }

    async fn save_pre_key(
        &mut self,
        prekey_id: PreKeyId,
        record: &PreKeyRecord,
        _ctx: Context,
    ) -> SignalResult<()> {
        // This overwrites old values, which matches Java behavior, but is it correct?
        let key = u32::from(prekey_id).to_le_bytes();
        let value = record.serialize()?;
        self.0
            .insert(key, value)
            .map_err(|err| sled_to_signal_error("save_pre_key", err))?;
        Ok(())
    }

    async fn remove_pre_key(&mut self, prekey_id: PreKeyId, _ctx: Context) -> SignalResult<()> {
        // If `prekey_id` does not exist this silently does nothing
        let key = u32::from(prekey_id).to_le_bytes();
        self.0
            .remove(key)
            .map_err(|err| sled_to_signal_error("remove_pre_key", err))?;
        Ok(())
    }
}
