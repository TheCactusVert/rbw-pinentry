use std::{
    io::{self, BufRead, Write},
    str::FromStr,
};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use keyring::Entry;
use notify_rust::Notification;
use regex::Regex;
use rpassword::prompt_password;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Pinentry {
    SETTITLE(String),
    SETDESC(String),
    SETPROMPT(String),
    SETERROR(String),
    GETPIN,
    BYE,
    UNKNOWN,
}

impl FromStr for Pinentry {
    type Err = regex::Error;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
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

fn pinentry(entry: &Entry) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut handle = stdout.lock();

    let mut prompt: Option<String> = None;

    for line in stdin.lock().lines() {
        match Pinentry::from_str(&line.unwrap())? {
            Pinentry::SETTITLE(_arg) => {
                print_ok(&mut handle)?;
            }
            Pinentry::SETDESC(_arg) => {
                print_ok(&mut handle)?;
            }
            Pinentry::SETPROMPT(arg) => {
                prompt = Some(arg);
                print_ok(&mut handle)?;
            }
            Pinentry::GETPIN => match prompt.as_ref().map(|p| p.as_str()) {
                Some("Master Password") => {
                    match entry.get_password() {
                        Ok(password) => {
                            print_password(&mut handle, &password)?;
                            print_ok(&mut handle)?;
                        }
                        Err(_e) => {
                            Notification::new()
                                .summary("rbw - Master password doesn't exist")
                                .body("Use 'rbw-pinentry store' to create a new entry.")
                                .show()?;
                            print_error(&mut handle, "1 no master password")?;
                        }
                    };
                }
                Some(_) => {
                    print_error(&mut handle, "2 unknown prompt")?;
                }
                None => {
                    print_error(&mut handle, "3 no prompt")?;
                }
            },
            Pinentry::BYE => {
                break;
            }
            Pinentry::SETERROR(_e) => {
                Notification::new()
                    .summary("rbw - Error")
                    .body(
                        "Master password is incorrect. Use 'rbw-pinentry store' to edit the entry.",
                    )
                    .show()?;
                print_error(&mut handle, "4 notification sent")?;
            }
            _ => {
                print_error(&mut handle, "5 unknown command")?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let user = &args
        .profile
        .and_then(|p| Some(format!("{p}-{SUFFIX}")))
        .unwrap_or(SUFFIX.to_string());

    let entry = Entry::new("rbw", &user)?;

    match args.command {
        Some(Commands::Store) => store(&entry),
        Some(Commands::Lookup) => lookup(&entry),
        Some(Commands::Clear) => clear(&entry),
        None => pinentry(&entry),
    }
}
