use std::{env::args, fs::File};

use crate::error::{Error, Result};

mod credentials;
mod provision;
mod register_device;

pub use credentials::Credentials;

pub fn register() -> Result<()> {
    let mut args = args();
    assert!(
        args.len() >= 2 && args.len() <= 3,
        "Expected 1-2 arguments: <device name>[ <output file>]. Received: {:?}",
        args
    );

    let _exec = args.next().expect("Path to executable.");
    let name = args.next().expect("Device name.");
    let file = match args.next() {
        Some(path) => Some(File::create(path).map_err(|err| Error::IoError(err))?),
        None => None,
    };

    let provision_message = provision::get_provision_message()?;
    let creds = register_device::register_device(provision_message, &name)?;

    if let Some(file) = file {
        eprintln!("Registered! Storing credentials to specified file.");
        serde_json::to_writer(file, &creds)?;
    } else {
        eprintln!("Registered! Dumping credentials to stdout: (safe to pipe)");
        println!("{}", serde_json::to_string(&creds)?);
    }
    Ok(())
}
