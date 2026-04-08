use std::{
    io::{self, BufRead, Write},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use keyring::Entry;
use regex::Regex;
use systemd::unit::escape_name;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum PinentryArgs {
    SETTITLE { title: String },
    SETDESC { desc: String },
    SETPROMPT { prompt: String },
    SETERROR { error: String },
    GETPIN,
    BYE,
    UNKNOWN,
}

impl FromStr for PinentryArgs {
    type Err = regex::Error;

    fn from_str(input: &str) -> std::result::Result<PinentryArgs, Self::Err> {
        let re = Regex::new(r"^(\w*) ?(.*)?$")?;
        re.captures(input);

        if let Some(code) = re.captures(input) {
            match &code[1] {
                "SETTITLE" => Ok(PinentryArgs::SETTITLE {
                    title: code[2].to_string(),
                }),
                "SETDESC" => Ok(PinentryArgs::SETDESC {
                    desc: code[2].to_string(),
                }),
                "SETPROMPT" => Ok(PinentryArgs::SETPROMPT {
                    prompt: code[2].to_string(),
                }),
                "SETERROR" => Ok(PinentryArgs::SETERROR {
                    error: code[2].to_string(),
                }),
                "GETPIN" => Ok(PinentryArgs::GETPIN),
                "BYE" => Ok(PinentryArgs::BYE),
                _ => Ok(PinentryArgs::UNKNOWN),
            }
        } else {
            panic!(); // TODO
        }
    }
}

#[derive(Subcommand, Default)]
enum Commands {
    Store {
        password: String,
    },
    Lookup,
    Clear,
    #[default]
    Pinentry,
}

/// Pinentry for rbw using system keyring
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, env = "RBW_PROFILE")]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Timeout waiting for input after this many seconds
    #[arg(short = 'o', long = "timeout", default_value = "0")]
    timeout: u64, // TODO Do something with this ?

    /// Set the tty terminal node name
    #[arg(short = 'T', long = "ttyname")]
    ttyname: Option<String>, // TODO Do something with this ?
}

static SUFFIX: &'static str = "passwd";

fn print_ok<T: Write>(handle: &mut T) {
    handle.write(b"OK\n");
}

fn print_password<T: Write>(handle: &mut T, entry: &Entry) -> keyring::Result<()> {
    let password = entry.get_password()?;
    write!(handle, "D {password}\n");
    Ok(())
}

fn print_error<T: Write>(handle: &mut T, message: &str) {
    write!(handle, "ERR {message}\n");
}

fn main() -> keyring::Result<()> {
    let args = Cli::parse();

    let user = escape_name(
        &args
            .profile
            .and_then(|p| Some(format!("{p}/{SUFFIX}")))
            .unwrap_or(SUFFIX.to_string()),
    );

    let entry = Entry::new("rbw", &user)?;

    match args.command.unwrap_or_default() {
        Commands::Store { password } => {
            entry.set_password(&password)?;
        }
        Commands::Lookup => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            print_password(&mut handle, &entry)?;
        }
        Commands::Clear => {
            entry.delete_credential()?;
        }
        Commands::Pinentry => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            for line in stdin.lock().lines() {
                match PinentryArgs::from_str(&line.unwrap()).unwrap() {
                    PinentryArgs::SETTITLE { title } => {
                        print_ok(&mut handle);
                    }
                    PinentryArgs::SETDESC { desc } => {
                        print_ok(&mut handle);
                    }
                    PinentryArgs::SETPROMPT { prompt } => {
                        print_ok(&mut handle);
                    }
                    PinentryArgs::GETPIN => {
                        print_password(&mut handle, &entry)?;
                        print_ok(&mut handle);
                    }
                    PinentryArgs::BYE => {
                        break;
                    }
                    _ => {
                        print_error(&mut handle, "Unknown command");
                    }
                }
            }
        }
    }

    Ok(())
}
