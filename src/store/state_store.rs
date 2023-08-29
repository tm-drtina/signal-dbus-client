use std::convert::TryInto;
use std::path::Path;

use async_trait::async_trait;
use libsignal_protocol::error::Result as SignalResult;
use libsignal_protocol::{
    Direction, IdentityKey, IdentityKeyPair, IdentityKeyStore, KyberPreKeyId, KyberPreKeyRecord,
    KyberPreKeyStore, PreKeyId, PreKeyRecord, PreKeyStore, ProtocolAddress, ProtocolStore,
    SessionRecord, SessionStore, SignedPreKeyId, SignedPreKeyRecord, SignedPreKeyStore,
};

use crate::error::Result;

use super::{SledIdentityStore, SledPreKeyStore, SledSessionStore, SledSignedPreKeyStore, SledKyberPreKeyStore};

pub(crate) struct SledStateStore {
    pub(crate) session_store: SledSessionStore,
    pub(crate) pre_key_store: SledPreKeyStore,
    pub(crate) signed_pre_key_store: SledSignedPreKeyStore,
    pub(crate) identity_store: SledIdentityStore,
    pub(crate) kyber_pre_key_store: SledKyberPreKeyStore,
}

impl SledStateStore {
    pub(crate) fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let db = &sled::open(data_dir)?;

        Ok(Self {
            session_store: db.try_into()?,
            pre_key_store: db.try_into()?,
            signed_pre_key_store: db.try_into()?,
            identity_store: db.try_into()?,
            kyber_pre_key_store: db.try_into()?,
        })
    }

    pub(crate) fn api_username(&self) -> Result<String> {
        self.identity_store.get_api_user()
    }

    pub(crate) fn api_password(&self) -> Result<String> {
        self.identity_store.get_api_pass()
    }

    pub(crate) fn register_new_account(
        &self,
        identity_key_pair: IdentityKeyPair,
        registration_id: u32,
        address: ProtocolAddress,
        api_pass: String,
    ) -> Result<()> {
        self.identity_store.register_new_account(
            identity_key_pair,
            registration_id,
            address,
            api_pass,
        )
    }
}

#[async_trait(?Send)]
impl SessionStore for SledStateStore {
    async fn load_session(&self, address: &ProtocolAddress) -> SignalResult<Option<SessionRecord>> {
        self.session_store.load_session(address).await
    }

    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: &SessionRecord,
    ) -> SignalResult<()> {
        self.session_store.store_session(address, record).await
    }
}

#[async_trait(?Send)]
impl PreKeyStore for SledStateStore {
    async fn get_pre_key(&self, prekey_id: PreKeyId) -> SignalResult<PreKeyRecord> {
        self.pre_key_store.get_pre_key(prekey_id).await
    }

    async fn save_pre_key(
        &mut self,
        prekey_id: PreKeyId,
        record: &PreKeyRecord,
    ) -> SignalResult<()> {
        self.pre_key_store.save_pre_key(prekey_id, record).await
    }

    async fn remove_pre_key(&mut self, prekey_id: PreKeyId) -> SignalResult<()> {
        self.pre_key_store.remove_pre_key(prekey_id).await
    }
}

#[async_trait(?Send)]
impl SignedPreKeyStore for SledStateStore {
    async fn get_signed_pre_key(
        &self,
        signed_prekey_id: SignedPreKeyId,
    ) -> SignalResult<SignedPreKeyRecord> {
        self.signed_pre_key_store
            .get_signed_pre_key(signed_prekey_id)
            .await
    }

    async fn save_signed_pre_key(
        &mut self,
        signed_prekey_id: SignedPreKeyId,
        record: &SignedPreKeyRecord,
    ) -> SignalResult<()> {
        self.signed_pre_key_store
            .save_signed_pre_key(signed_prekey_id, record)
            .await
    }
}

#[async_trait(?Send)]
impl IdentityKeyStore for SledStateStore {
    async fn get_identity_key_pair(&self) -> SignalResult<IdentityKeyPair> {
        self.identity_store.get_identity_key_pair().await
    }

    async fn get_local_registration_id(&self) -> SignalResult<u32> {
        self.identity_store.get_local_registration_id().await
    }

    async fn is_trusted_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        direction: Direction,
    ) -> SignalResult<bool> {
        self.identity_store
            .is_trusted_identity(address, identity, direction)
            .await
    }

    async fn get_identity(&self, address: &ProtocolAddress) -> SignalResult<Option<IdentityKey>> {
        self.identity_store.get_identity(address).await
    }

    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> SignalResult<bool> {
        self.identity_store.save_identity(address, identity).await
    }
}

#[async_trait(?Send)]
impl KyberPreKeyStore for SledStateStore {
    #[must_use]
    async fn get_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> SignalResult<KyberPreKeyRecord> {
        self.kyber_pre_key_store.get_kyber_pre_key(kyber_prekey_id).await
    }

    #[must_use]
    async fn save_kyber_pre_key(
        &mut self,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKeyRecord,
    ) -> SignalResult<()> {
        self.kyber_pre_key_store.save_kyber_pre_key(kyber_prekey_id, record).await
    }

    #[must_use]
    async fn mark_kyber_pre_key_used(
        &mut self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> SignalResult<()> {
        self.kyber_pre_key_store.mark_kyber_pre_key_used(kyber_prekey_id).await
    }
}

impl ProtocolStore for SledStateStore {}
