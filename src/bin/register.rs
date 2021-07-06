extern crate signal_dbus_client;

use signal_dbus_client::{register_entrypoint, error::Result};

fn main() -> Result<()> {
    register_entrypoint()?;
    Ok(())
}