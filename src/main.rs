use std::fs::DirBuilder;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use signal_dbus_client::error::Result;
use signal_dbus_client::{register, send_message};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(
    propagate_version = true,
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[arg(
        long,
        short,
        value_name = "DIRECTORY",
        help = "Sets a custom data directory"
    )]
    data_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Register new sub-device")]
    Register {
        #[arg(help = "Sets the name of newly registered sub-device")]
        name: String,
    },
    #[command(about = "Sends message to specified recipient")]
    Send {
        #[arg(help = "Recipient of the message. Either E164 telephone format or UUID")]
        recipient: String,
        message: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let data_dir = if let Some(path) = cli.data_dir {
        test_writeable_directory(&path)?;
        path
    } else {
        get_default_data_dir()?
    };

    match cli.command {
        Commands::Register { name } => register(data_dir, &name).await,
        Commands::Send { recipient, message } => send_message(data_dir, &recipient, &message).await,
    }
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
