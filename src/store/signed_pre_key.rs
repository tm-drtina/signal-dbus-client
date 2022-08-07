use std::convert::TryFrom;

use async_trait::async_trait;
use libsignal_protocol::error::Result as SignalResult;
use libsignal_protocol::{
    Context, SignalProtocolError, SignedPreKeyId, SignedPreKeyRecord, SignedPreKeyStore,
};
use sled::{Db, Tree};

use super::utils::sled_to_signal_error;

pub(crate) struct SledSignedPreKeyStore(Tree);

impl TryFrom<&Db> for SledSignedPreKeyStore {
    type Error = sled::Error;
    fn try_from(db: &Db) -> Result<Self, Self::Error> {
        Ok(Self(db.open_tree("signed-pre-keys")?))
    }
}

#[async_trait(?Send)]
impl SignedPreKeyStore for SledSignedPreKeyStore {
    async fn get_signed_pre_key(
        &self,
        signed_prekey_id: SignedPreKeyId,
        _ctx: Context,
    ) -> SignalResult<SignedPreKeyRecord> {
        let key = u32::from(signed_prekey_id).to_le_bytes();
        match self.0.get(key) {
            Ok(Some(bytes)) => SignedPreKeyRecord::deserialize(&bytes),
            Ok(None) => Err(SignalProtocolError::InvalidPreKeyId),
            Err(err) => Err(sled_to_signal_error("get_signed_pre_key", err)),
        }
    }

    async fn save_signed_pre_key(
        &mut self,
        signed_prekey_id: SignedPreKeyId,
        record: &SignedPreKeyRecord,
        _ctx: Context,
    ) -> SignalResult<()> {
        // This overwrites old values, which matches Java behavior, but is it correct?
        let key = u32::from(signed_prekey_id).to_le_bytes();
        let value = record.serialize()?;
        self.0
            .insert(key, value)
            .map_err(|err| sled_to_signal_error("save_signed_pre_key", err))?;
        Ok(())
    }
}
