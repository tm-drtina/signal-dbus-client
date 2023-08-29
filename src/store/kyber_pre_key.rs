use std::convert::TryFrom;

use async_trait::async_trait;
use libsignal_protocol::error::Result as SignalResult;
use libsignal_protocol::{
    GenericSignedPreKey, KyberPreKeyId, KyberPreKeyRecord, KyberPreKeyStore, SignalProtocolError,
};
use sled::{Db, Tree};

use super::utils::sled_to_signal_error;

pub(crate) struct SledKyberPreKeyStore(Tree);

impl TryFrom<&Db> for SledKyberPreKeyStore {
    type Error = sled::Error;
    fn try_from(db: &Db) -> Result<Self, Self::Error> {
        Ok(Self(db.open_tree("kyber-pre-keys")?))
    }
}
#[async_trait(?Send)]
impl KyberPreKeyStore for SledKyberPreKeyStore {
    #[must_use]
    async fn get_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> SignalResult<KyberPreKeyRecord> {
        let key = u32::from(kyber_prekey_id).to_le_bytes();
        match self.0.get(key) {
            Ok(Some(bytes)) => KyberPreKeyRecord::deserialize(&bytes),
            Ok(None) => Err(SignalProtocolError::InvalidPreKeyId),
            Err(err) => Err(sled_to_signal_error("get_kyber_pre_key", err)),
        }
    }

    #[must_use]
    async fn save_kyber_pre_key(
        &mut self,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKeyRecord,
    ) -> SignalResult<()> {
        // This overwrites old values, which matches Java behavior, but is it correct?
        let key = u32::from(kyber_prekey_id).to_le_bytes();
        let value = record.serialize()?;
        self.0
            .insert(key, value)
            .map_err(|err| sled_to_signal_error("save_kyber_pre_key", err))?;
        Ok(())
    }

    #[must_use]
    async fn mark_kyber_pre_key_used(
        &mut self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> SignalResult<()> {
        // If `prekey_id` does not exist this silently does nothing
        let key = u32::from(kyber_prekey_id).to_le_bytes();
        self.0
            .remove(key)
            .map_err(|err| sled_to_signal_error("remove_kyber_pre_key", err))?;
        Ok(())
    }
}
