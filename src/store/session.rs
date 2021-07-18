use std::convert::TryFrom;

use async_trait::async_trait;
use libsignal_protocol::error::Result as SignalResult;
use libsignal_protocol::{Context, ProtocolAddress, SessionRecord, SessionStore};
use sled::{Db, Tree};

use super::utils::{sled_to_signal_error, ProtocolAddressBytes};

pub(crate) struct SledSessionStore(Tree);

impl TryFrom<&Db> for SledSessionStore {
    type Error = sled::Error;
    fn try_from(db: &Db) -> Result<Self, Self::Error> {
        Ok(Self(db.open_tree("sessions")?))
    }
}

#[async_trait(?Send)]
impl SessionStore for SledSessionStore {
    async fn load_session<'s, 'a>(
        &'s self,
        address: &'a ProtocolAddress,
        _ctx: Context,
    ) -> SignalResult<Option<SessionRecord>> {
        let key = ProtocolAddressBytes::from(address);
        match self.0.get(key) {
            Ok(Some(bytes)) => SessionRecord::deserialize(&bytes).map(Some),
            Ok(None) => Ok(None),
            Err(err) => Err(sled_to_signal_error("load_session", err)),
        }
    }

    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: &SessionRecord,
        _ctx: Context,
    ) -> SignalResult<()> {
        let key = ProtocolAddressBytes::from(address);
        let value = record.serialize()?;
        self.0
            .insert(key, value)
            .map_err(|err| sled_to_signal_error("load_session", err))?;
        Ok(())
    }
}