mod pinentry;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use keyring_core::Entry;
use rpassword::prompt_password;
use zbus_secret_service_keyring_store::Store;

#[derive(Subcommand)]
enum Commands {
    Store,
    Lookup,
    Clear,
}

/// Pinentry for rbw using system keyring
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, env = "RBW_PROFILE")]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Set the X display
    #[arg(short = 'D', long = "display")]
    display: Option<String>, // TODO Do something with this ?

    /// Set the tty terminal node name
    #[arg(short = 'T', long = "ttyname")]
    ttyname: Option<String>, // TODO Do something with this ?

    /// Timeout waiting for input after this many seconds
    #[arg(short = 'o', long = "timeout", default_value = "0")]
    timeout: u64, // TODO Do something with this ?

    /// Grab keyboard only while window is focused
    #[arg(short = 'g', long = "no-global-grab", default_value = "false")]
    no_global_grab: bool, // TODO Do something with this ?
}

fn store(entry: &Entry) -> Result<()> {
    let password = prompt_password("Your master password: ")?;
    entry.set_password(&password)?;
    Ok(())
}

fn lookup(entry: &Entry) -> Result<()> {
    match entry.get_password() {
        Ok(password) => {
            println!("{password}");
            Ok(())
        }
        Err(e) => {
            println!("rbw-pinentry - Master password doesn't exist");
            println!("Use 'rbw-pinentry store' to create a new entry");
            Err(anyhow!(e))
        }
    }
}

fn clear(entry: &Entry) -> Result<()> {
    entry.delete_credential()?;
    Ok(())
}

static SUFFIX: &'static str = "passwd";

fn main() -> Result<()> {
    let args = Cli::parse();
    keyring_core::set_default_store(Store::new()?);

    let profile = &args
        .profile
        .and_then(|p| Some(format!("{p}-{SUFFIX}")))
        .unwrap_or(SUFFIX.to_string());

    let entry = Entry::new("rbw", &profile)?;

    let ret = match args.command {
        Some(Commands::Store) => store(&entry),
        Some(Commands::Lookup) => lookup(&entry),
        Some(Commands::Clear) => clear(&entry),
        None => pinentry::exec(&entry),
    };

    keyring_core::unset_default_store();

    ret
}
