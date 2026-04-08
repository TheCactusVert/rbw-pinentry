use std::{
    io::{self, BufRead, Write},
    process::Output,
    str::FromStr,
};

use clap::{Parser, Subcommand};
use keyring::Entry;
use regex::Regex;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum PinentryArgs {
    SETTITLE { arg: String },
    SETDESC { arg: String },
    SETPROMPT { arg: String },
    SETERROR { arg: String },
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
                    arg: code[2].to_string(),
                }),
                "SETDESC" => Ok(PinentryArgs::SETDESC {
                    arg: code[2].to_string(),
                }),
                "SETPROMPT" => Ok(PinentryArgs::SETPROMPT {
                    arg: code[2].to_string(),
                }),
                "SETERROR" => Ok(PinentryArgs::SETERROR {
                    arg: code[2].to_string(),
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

fn print_ok<T: Write>(out: &mut T) {
    out.write(b"OK\n");
}

fn print_password<T: Write>(out: &mut T, entry: &Entry) -> keyring::Result<()> {
    let password = entry.get_password()?;
    write!(out, "D {password}\n");
    Ok(())
}

fn print_error<T: Write>(out: &mut T, message: &str) {
    write!(out, "ERR {message}\n");
}

fn main() -> keyring::Result<()> {
    let args = Cli::parse();

    let user = &args
        .profile
        .and_then(|p| Some(format!("{p}-{SUFFIX}")))
        .unwrap_or(SUFFIX.to_string());

    let entry = Entry::new("rbw", &user)?;

    match args.command.unwrap_or_default() {
        Commands::Store { password } => {
            entry.set_password(&password)?;
        }
        Commands::Lookup => {
            let password = entry.get_password()?;
            println!("{password}");
        }
        Commands::Clear => {
            entry.delete_credential()?;
        }
        Commands::Pinentry => {
            let stdin = io::stdin();
            let stdout = io::stdout();

            let mut handle = stdout.lock();

            let mut prompt: Option<String> = None;

            for line in stdin.lock().lines() {
                match PinentryArgs::from_str(&line.unwrap()).unwrap() {
                    PinentryArgs::SETTITLE { arg } => {
                        print_ok(&mut handle);
                    }
                    PinentryArgs::SETDESC { arg } => {
                        print_ok(&mut handle);
                    }
                    PinentryArgs::SETPROMPT { arg } => {
                        prompt = Some(arg);
                        print_ok(&mut handle);
                    }
                    PinentryArgs::GETPIN if prompt == Some("Master Password".to_string()) => {
                        print_password(&mut handle, &entry)?; // TODO fallback
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
