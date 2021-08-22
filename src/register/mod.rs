use std::path::PathBuf;

use rand::rngs::OsRng;

use crate::account::AccountManager;
use crate::common::ApiConfig;
use crate::error::Result;
use crate::store::SledStateStore;

mod credentials;
mod provision;
mod register_device;

pub async fn register(data_dir: PathBuf, name: &str) -> Result<()> {
    let api_config = ApiConfig::default();
    let csprng = &mut OsRng;

    let provision_message = provision::get_provision_message(&api_config).await?;
    eprintln!("Received provision message.");
    let creds = register_device::register_device(&api_config, provision_message, name).await?;
    eprintln!("Device registered successfuly.");

    let state_store = SledStateStore::new(&data_dir)?;
    state_store.register_new_account(
        creds.identity_key_pair,
        creds.registration_id,
        creds.address,
        creds.api_pass,
    )?;
    eprintln!("Stored credentials in state store.");

    let mut account_manager = AccountManager::with_store(state_store, csprng, &api_config)?;
    account_manager.initialize_pre_keys().await?;
    eprintln!("Initialized pre keys.");

    Ok(())
}
