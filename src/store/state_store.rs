use std::convert::TryInto;
use std::path::Path;

use async_trait::async_trait;
use libsignal_protocol::error::Result as SignalResult;
use libsignal_protocol::{
    Context, Direction, IdentityKey, IdentityKeyPair, IdentityKeyStore, PreKeyId, PreKeyRecord,
    PreKeyStore, ProtocolAddress, ProtocolStore, SessionRecord, SessionStore, SignedPreKeyId,
    SignedPreKeyRecord, SignedPreKeyStore,
};

use crate::error::Result;

use super::{SledIdentityStore, SledPreKeyStore, SledSessionStore, SledSignedPreKeyStore};

pub(crate) struct SledStateStore {
    pub(crate) session_store: SledSessionStore,
    pub(crate) pre_key_store: SledPreKeyStore,
    pub(crate) signed_pre_key_store: SledSignedPreKeyStore,
    pub(crate) identity_store: SledIdentityStore,
}

impl SledStateStore {
    pub(crate) fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let db = &sled::open(data_dir)?;

        Ok(Self {
            session_store: db.try_into()?,
            pre_key_store: db.try_into()?,
            signed_pre_key_store: db.try_into()?,
            identity_store: db.try_into()?,
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
    async fn load_session(
        &self,
        address: &ProtocolAddress,
        ctx: Context,
    ) -> SignalResult<Option<SessionRecord>> {
        self.session_store.load_session(address, ctx).await
    }

    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: &SessionRecord,
        ctx: Context,
    ) -> SignalResult<()> {
        self.session_store.store_session(address, record, ctx).await
    }
}

#[async_trait(?Send)]
impl PreKeyStore for SledStateStore {
    async fn get_pre_key(&self, prekey_id: PreKeyId, ctx: Context) -> SignalResult<PreKeyRecord> {
        self.pre_key_store.get_pre_key(prekey_id, ctx).await
    }

    async fn save_pre_key(
        &mut self,
        prekey_id: PreKeyId,
        record: &PreKeyRecord,
        ctx: Context,
    ) -> SignalResult<()> {
        self.pre_key_store
            .save_pre_key(prekey_id, record, ctx)
            .await
    }

    async fn remove_pre_key(&mut self, prekey_id: PreKeyId, ctx: Context) -> SignalResult<()> {
        self.pre_key_store.remove_pre_key(prekey_id, ctx).await
    }
}

#[async_trait(?Send)]
impl SignedPreKeyStore for SledStateStore {
    async fn get_signed_pre_key(
        &self,
        signed_prekey_id: SignedPreKeyId,
        ctx: Context,
    ) -> SignalResult<SignedPreKeyRecord> {
        self.signed_pre_key_store
            .get_signed_pre_key(signed_prekey_id, ctx)
            .await
    }

    async fn save_signed_pre_key(
        &mut self,
        signed_prekey_id: SignedPreKeyId,
        record: &SignedPreKeyRecord,
        ctx: Context,
    ) -> SignalResult<()> {
        self.signed_pre_key_store
            .save_signed_pre_key(signed_prekey_id, record, ctx)
            .await
    }
}

#[async_trait(?Send)]
impl IdentityKeyStore for SledStateStore {
    async fn get_identity_key_pair(&self, ctx: Context) -> SignalResult<IdentityKeyPair> {
        self.identity_store.get_identity_key_pair(ctx).await
    }

    async fn get_local_registration_id(&self, ctx: Context) -> SignalResult<u32> {
        self.identity_store.get_local_registration_id(ctx).await
    }

    async fn is_trusted_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        direction: Direction,
        ctx: Context,
    ) -> SignalResult<bool> {
        self.identity_store
            .is_trusted_identity(address, identity, direction, ctx)
            .await
    }

    async fn get_identity(
        &self,
        address: &ProtocolAddress,
        ctx: Context,
    ) -> SignalResult<Option<IdentityKey>> {
        self.identity_store.get_identity(address, ctx).await
    }

    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        ctx: Context,
    ) -> SignalResult<bool> {
        self.identity_store
            .save_identity(address, identity, ctx)
            .await
    }
}

impl ProtocolStore for SledStateStore {}
