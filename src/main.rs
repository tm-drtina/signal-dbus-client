use std::fs::DirBuilder;
use std::path::{Path, PathBuf};

use clap::{App, AppSettings, Arg, SubCommand};
use signal_dbus_client::error::Result;
use signal_dbus_client::{register, send_message};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let matches = App::new("SignalApp client")
        .version("0.1.0")
        .author("Tomas Drtina <tm.drtina@gmail.com>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("data-dir")
                .short("d")
                .long("data-dir")
                .value_name("DIRECTORY")
                .help("Sets a custom data directory")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("register")
                .about("Register new sub-device")
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .index(1)
                        .required(true)
                        .help("Sets the name of newly registered sub-device"),
                ),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Sends message to specified recipient")
                .arg(
                    Arg::with_name("recipient")
                        .short("r")
                        .long("recipient")
                        .index(1)
                        .required(true)
                        .help("Recipient of the message. Either E164 telephone format or UUID"),
                )
                .arg(
                    Arg::with_name("message")
                        .short("m")
                        .long("message")
                        .index(2)
                        .required(true)
                        .help("Message"),
                ),
        )
        .get_matches();

    let data_dir = if let Some(path) = matches.value_of("data-dir") {
        let path = PathBuf::from(path);

        test_writeable_directory(&path)?;
        path
    } else {
        get_default_data_dir()?
    };

    // TODO: improve when clap 3 is out
    match matches.subcommand {
        Some(subcommand) if subcommand.name == "register" => {
            let name = subcommand
                .matches
                .value_of("name")
                .expect("Name is required arg.");
            register(data_dir, name).await?;
        }
        Some(subcommand) if subcommand.name == "send" => {
            let recipient = subcommand
                .matches
                .value_of("recipient")
                .expect("Recipient is required arg.");
            let message = subcommand
                .matches
                .value_of("message")
                .expect("Message is required arg.");
            send_message(data_dir, recipient, message).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn test_writeable_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        let mut dir_builder = DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            dir_builder.mode(0o700);
        }
        dir_builder.create(&path)?;
    }
    if !path.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Expected data dir to be directory",
        )
        .into());
    }
    if path.metadata()?.permissions().readonly() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Data dir is readonly",
        )
        .into());
    }
    Ok(())
}

fn get_default_data_dir() -> Result<PathBuf> {
    let data_sub_path = "signal-client";

    if let Some(mut path) = dirs::data_dir() {
        path.push(data_sub_path);
        match test_writeable_directory(&path) {
            Ok(()) => return Ok(path),
            Err(err) => eprintln!(
                "Cannot access data directory. Falling back to current dir. Error: {}",
                err
            ),
        }
    }

    let path = std::env::current_dir()?;
    test_writeable_directory(&path)?;
    Ok(path)
}
