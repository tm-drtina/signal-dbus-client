use std::path::PathBuf;

use rand::rngs::OsRng;

use crate::account::AccountManager;
use crate::{common::ApiConfig, error::Result};

pub async fn send_message(data_dir: PathBuf, recipient: &str, message: &str) -> Result<()> {
    let csprng = &mut OsRng;
    let api_config = ApiConfig::default();
    let account_manager = AccountManager::new(data_dir, csprng, &api_config)?;

    account_manager.send_message(recipient, message).await?;

    Ok(())
}
