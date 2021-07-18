use std::convert::TryInto;
use std::path::PathBuf;

use hyper::Method;
use libsignal_protocol::{
    message_encrypt, process_prekey_bundle, DeviceId, IdentityKeyStore, PreKeyBundle, PreKeyStore,
    ProtocolAddress, SignedPreKeyStore,
};
use rand::{CryptoRng, Rng};

use crate::account::messages::{MessagesWrapper, SendMetadata};
use crate::account::pre_keys::DeviceKeys;
use crate::common::{ApiConfig, ApiPath};
use crate::error::Result;
use crate::store::SledStateStore;
use crate::utils::HttpClient;

use super::pre_keys::{generate_pre_keys, generate_signed_pre_key, PreKeyState};

pub(crate) struct AccountManager<'r, R: Rng + CryptoRng> {
    http_client: HttpClient,
    state: SledStateStore,
    csprng: &'r mut R,
}

impl<'r, R: Rng + CryptoRng> AccountManager<'r, R> {
    pub(crate) fn new(
        data_dir: PathBuf,
        csprng: &'r mut R,
        api_config: &ApiConfig,
    ) -> Result<Self> {
        let state = SledStateStore::new(data_dir)?;
        Self::with_store(state, csprng, api_config)
    }

    pub(crate) fn with_store(
        state: SledStateStore,
        csprng: &'r mut R,
        api_config: &ApiConfig,
    ) -> Result<Self> {
        let username = state.api_username()?;
        let password = state.api_password()?;
        let http_client = HttpClient::new(&username, &password, api_config)?;

        Ok(Self {
            http_client,
            state,
            csprng,
        })
    }

    pub async fn initialize_pre_keys(&mut self) -> Result<()> {
        let pre_keys = generate_pre_keys(100, self.csprng);
        let identity_key_pair = self.state.get_identity_key_pair(None).await?;
        let signed_pre_key = generate_signed_pre_key(&identity_key_pair, 1, self.csprng)?;

        for pre_key in pre_keys.iter() {
            self.state
                .save_pre_key(pre_key.id()?, pre_key, None)
                .await?;
        }
        self.state
            .save_signed_pre_key(signed_pre_key.id()?, &signed_pre_key, None)
            .await?;

        let pre_key_state = PreKeyState::new(
            *identity_key_pair.identity_key(),
            &pre_keys,
            &signed_pre_key,
        )?;

        self.register_prekeys(pre_key_state).await?;

        Ok(())
    }

    pub async fn create_session(
        &mut self,
        recipient: &str,
        device_id: Option<DeviceId>,
    ) -> Result<Vec<(ProtocolAddress, u32)>> {
        let device_id = &device_id.map_or(String::from("*"), |x| x.to_string());

        let response: DeviceKeys = self
            .http_client
            .send(
                Method::GET,
                ApiPath::GetSessionKey {
                    recipient,
                    device_id,
                },
            )
            .await?
            .json()
            .await?;

        let bundles: Vec<PreKeyBundle> = response.try_into()?;
        let mut addrs = Vec::with_capacity(bundles.len());

        for bundle in bundles {
            let remote_address = ProtocolAddress::new(
                recipient.to_string(),
                bundle.device_id().expect("Impl doesn't return Err"),
            );
            process_prekey_bundle(
                &remote_address,
                &mut self.state.session_store,
                &mut self.state.identity_store,
                &bundle,
                self.csprng,
                None,
            )
            .await?;
            addrs.push((remote_address, bundle.registration_id()?));
        }

        Ok(addrs)
    }

    async fn register_prekeys(&self, pre_key_state: PreKeyState) -> Result<()> {
        self.http_client
            .send_json(Method::PUT, ApiPath::PreKeys, &pre_key_state)
            .await?;

        Ok(())
    }

    pub async fn send_message<S1: Into<String>, S2: Into<String>>(
        &mut self,
        recipient: S1,
        message: S2,
    ) -> Result<()> {
        let message: String = message.into();
        let recipient: String = recipient.into();

        let addrs = self.create_session(&recipient, None).await?;

        let mut send_metadata = Vec::<SendMetadata>::with_capacity(addrs.len());
        for (addr, registration_id) in addrs.into_iter() {
            let ciphertext_message = message_encrypt(
                message.as_bytes(),
                &addr,
                &mut self.state.session_store,
                &mut self.state.identity_store,
                None,
            )
            .await?;

            send_metadata.push(SendMetadata::new(
                ciphertext_message,
                addr.device_id(),
                registration_id,
            ));
        }

        let body = MessagesWrapper::new(recipient.clone(), send_metadata);

        // FIXME: this returns 400
        let response = self
            .http_client
            .send_json(
                Method::PUT,
                ApiPath::SendMessage {
                    recipient: &recipient,
                },
                &body,
            )
            .await?
            .text()
            .await?;

        eprintln!("{}", response);

        Ok(())
    }
}
