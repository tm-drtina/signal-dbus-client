use std::convert::TryInto;
use std::path::PathBuf;
use std::time::SystemTime;

use libsignal_protocol::{
    message_encrypt, process_prekey_bundle, DeviceId, IdentityKeyStore, PreKeyBundle, PreKeyStore,
    ProtocolAddress, SessionStore, SignedPreKeyId, SignedPreKeyStore,
};
use rand::{CryptoRng, Rng};
use reqwest::Method;

use crate::account::messages::{
    MessageResponse200, MessageResponse409, MessagesWrapper, SendMetadata,
};
use crate::account::pre_keys::DeviceKeys;
use crate::common::{ApiConfig, ApiPath};
use crate::error::{Error, Result};
use crate::store::SledStateStore;
use crate::utils::{Body, HttpClient};

use super::pre_keys::{generate_pre_keys, generate_signed_pre_key, PreKeyState};

pub(crate) struct AccountManager<'r, R: Rng + CryptoRng + Clone> {
    http_client: HttpClient,
    state: SledStateStore,
    csprng: &'r mut R,
}

impl<'r, R: Rng + CryptoRng + Clone> AccountManager<'r, R> {
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
        let identity_key_pair = self.state.get_identity_key_pair().await?;
        let signed_pre_key_id = SignedPreKeyId::from(1);
        let signed_pre_key =
            generate_signed_pre_key(&identity_key_pair, signed_pre_key_id, self.csprng)?;

        for pre_key in pre_keys.iter() {
            self.state.save_pre_key(pre_key.id()?, pre_key).await?;
        }
        self.state
            .save_signed_pre_key(signed_pre_key_id, &signed_pre_key)
            .await?;

        let pre_key_state = PreKeyState::new(
            *identity_key_pair.identity_key(),
            &pre_keys,
            &signed_pre_key,
        )?;

        self.register_prekeys(pre_key_state).await?;

        Ok(())
    }

    pub async fn create_sessions(
        &self,
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
                Body::empty(),
            )
            .await?
            .json()
            .await?;

        eprintln!("{:?}", response);

        let bundles: Vec<PreKeyBundle> = response.try_into()?;
        let mut addrs = Vec::with_capacity(bundles.len());

        for bundle in bundles {
            let remote_address = ProtocolAddress::new(
                recipient.to_string(),
                bundle.device_id().expect("Impl doesn't return Err"),
            );

            let mut session_store = self.state.session_store.clone();
            let mut identity_store = self.state.identity_store.clone();
            let mut csprng = self.csprng.clone();

            process_prekey_bundle(
                &remote_address,
                &mut session_store,
                &mut identity_store,
                &bundle,
                SystemTime::now(),
                &mut csprng,
            )
            .await?;
            addrs.push((remote_address, bundle.registration_id()?));
        }

        Ok(addrs)
    }

    async fn register_prekeys(&self, pre_key_state: PreKeyState) -> Result<()> {
        self.http_client
            .send(Method::PUT, ApiPath::PreKeys, Body::Json(&pre_key_state))
            .await?;

        Ok(())
    }

    async fn load_sessions(&self, recipient: &str) -> Result<Vec<(ProtocolAddress, u32)>> {
        self.state
            .session_store
            .load_sessions_by_prefix(recipient)
            .await?
            .into_iter()
            .map(|(addr, session)| Ok((addr, session.remote_registration_id()?)))
            .collect()
    }

    async fn load_or_create_sessions(
        &self,
        recipient: &str,
    ) -> Result<Vec<(ProtocolAddress, u32)>> {
        let addrs = self.load_sessions(recipient).await?;
        if addrs.is_empty() {
            self.create_sessions(recipient, None).await
        } else {
            Ok(addrs)
        }
    }

    pub async fn send_message(&self, recipient: &str, message: &str) -> Result<()> {
        loop {
            let addrs = self.load_or_create_sessions(recipient).await?;

            let mut send_metadata = Vec::with_capacity(addrs.len());
            for (addr, registration_id) in addrs.into_iter() {
                // Clone is cheap, since our store is just a wrapped Arc.
                // This way we don't require &mut self and &self is enough.
                let mut session_store = self.state.session_store.clone();
                let mut identity_store = self.state.identity_store.clone();

                let ciphertext_message = message_encrypt(
                    message.as_bytes(),
                    &addr,
                    &mut session_store,
                    &mut identity_store,
                    SystemTime::now(),
                )
                .await?;

                send_metadata.push(SendMetadata::new(
                    ciphertext_message,
                    addr.device_id(),
                    registration_id,
                ));
            }

            let body = MessagesWrapper::new(send_metadata);

            let response_result = self
                .http_client
                .send(
                    Method::PUT,
                    ApiPath::SendMessage { recipient },
                    Body::Json(&body),
                )
                .await;

            let response: MessageResponse200 = match response_result {
                Ok(response) => response.json().await?,
                Err(Error::HttpError(status_code, value)) if status_code == 409 => {
                    let MessageResponse409 {
                        missing_devices,
                        extra_devices,
                    } = serde_json::from_str(&value)?;

                    for device_id in missing_devices {
                        self.create_sessions(recipient, Some(device_id)).await?;
                    }
                    for device_id in extra_devices {
                        let addr = ProtocolAddress::new(recipient.to_string(), device_id);
                        let mut session = self
                            .state
                            .load_session(&addr)
                            .await?
                            .expect("Extra session should be still present.");
                        // TODO: is archiving enough? Shouldn't we delete it?
                        session.archive_current_state()?;

                        // Clone is cheap, since our store is just a wrapped Arc.
                        // This way we don't require &mut self and &self is enough.
                        self.state
                            .session_store
                            .clone()
                            .store_session(&addr, &session)
                            .await?;
                    }
                    continue;
                }
                Err(err) => {
                    return Err(err);
                }
            };

            eprintln!("{:?}", response);

            // TODO: send sync message

            return Ok(());
        }
    }
}
