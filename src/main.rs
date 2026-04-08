use std::{
    io::{self, BufRead, Write},
    str::FromStr,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use keyring::Entry;
use regex::Regex;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PinentryArgs {
    SETTITLE(String),
    SETDESC(String),
    SETPROMPT(String),
    SETERROR(String),
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
                "SETTITLE" => Ok(Self::SETTITLE(code[2].to_string())),
                "SETDESC" => Ok(Self::SETDESC(code[2].to_string())),
                "SETPROMPT" => Ok(Self::SETPROMPT(code[2].to_string())),
                "SETERROR" => Ok(Self::SETERROR(code[2].to_string())),
                "GETPIN" => Ok(Self::GETPIN),
                "BYE" => Ok(Self::BYE),
                _ => Ok(Self::UNKNOWN),
            }
        } else {
            panic!(); // TODO
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    Store { password: String },
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

    /// Timeout waiting for input after this many seconds
    #[arg(short = 'o', long = "timeout", default_value = "0")]
    timeout: u64, // TODO Do something with this ?

    /// Set the tty terminal node name
    #[arg(short = 'T', long = "ttyname")]
    ttyname: Option<String>, // TODO Do something with this ?
}

static SUFFIX: &'static str = "passwd";

fn print_ok<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(out, "OK")
}

fn print_password<T: Write>(out: &mut T, password: &str) -> std::io::Result<()> {
    writeln!(out, "D {password}")
}

fn print_error<T: Write>(out: &mut T, message: &str) -> std::io::Result<()> {
    writeln!(out, "ERR {message}")
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let user = &args
        .profile
        .and_then(|p| Some(format!("{p}-{SUFFIX}")))
        .unwrap_or(SUFFIX.to_string());

    let entry = Entry::new("rbw", &user)?;

    match args.command {
        Some(Commands::Store { password }) => {
            entry.set_password(&password)?;
        }
        Some(Commands::Lookup) => {
            println!("{}", entry.get_password()?);
        }
        Some(Commands::Clear) => {
            entry.delete_credential()?;
        }
        None => {
            let stdin = io::stdin();
            let stdout = io::stdout();

            let mut handle = stdout.lock();

            let mut prompt: Option<String> = None;

            for line in stdin.lock().lines() {
                match PinentryArgs::from_str(&line.unwrap())? {
                    PinentryArgs::SETTITLE(_arg) => {
                        print_ok(&mut handle)?;
                    }
                    PinentryArgs::SETDESC(_arg) => {
                        print_ok(&mut handle)?;
                    }
                    PinentryArgs::SETPROMPT(arg) => {
                        prompt = Some(arg);
                        print_ok(&mut handle)?;
                    }
                    PinentryArgs::GETPIN if prompt == Some("Master Password".to_string()) => {
                        print_password(&mut handle, &entry.get_password()?)?; // TODO fallback if no password
                        print_ok(&mut handle)?;
                    } // TODO fallback if not master password
                    PinentryArgs::BYE => {
                        break;
                    }
                    _ => {
                        print_error(&mut handle, "Unknown command")?;
                    }
                }
            }
        }
    }

    Ok(())
}
