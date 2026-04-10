use std::{
    io::{self, BufRead, Write},
    str::FromStr,
};

use anyhow::Result;
use keyring::Entry;
use notify_rust::Notification;
use percent_encoding::{CONTROLS, percent_encode};
use regex::Regex;

enum Command {
    SETTITLE(String),
    SETDESC(String),
    SETPROMPT(String),
    SETERROR(String),
    GETPIN,
    BYE,
    UNKNOWN,
}

impl FromStr for Command {
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

fn write_introduction<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(
        out,
        "OK Pleased to meet you, process {}",
        std::process::id()
    )
}

fn write_ok<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(out, "OK")
}

fn write_password<T: Write>(out: &mut T, password: &str) -> std::io::Result<()> {
    let password = percent_encode(password.as_bytes(), CONTROLS);
    writeln!(out, "D {password}")
}

fn write_error<T: Write>(out: &mut T, message: &str) -> std::io::Result<()> {
    writeln!(out, "ERR {message}")
}

pub fn exec(entry: &Entry) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut handle = stdout.lock();

    let mut prompt: Option<String> = None;

    write_introduction(&mut handle)?;

    for line in stdin.lock().lines() {
        match Command::from_str(&line.unwrap())? {
            Command::SETTITLE(_arg) => {
                write_ok(&mut handle)?;
            }
            Command::SETDESC(_arg) => {
                write_ok(&mut handle)?;
            }
            Command::SETPROMPT(arg) => {
                prompt = Some(arg);
                write_ok(&mut handle)?;
            }
            Command::GETPIN => match prompt.as_ref().map(|p| p.as_str()) {
                Some("Master Password") => {
                    match entry.get_password() {
                        Ok(password) => {
                            write_password(&mut handle, &password)?;
                            write_ok(&mut handle)?;
                        }
                        Err(_e) => {
                            Notification::new()
                                .summary("rbw - Master password doesn't exist")
                                .body("Use 'rbw-pinentry store' to create a new entry.")
                                .show()?;
                            write_error(&mut handle, "1 no master password")?;
                        }
                    };
                }
                Some(_) => {
                    write_error(&mut handle, "2 unknown prompt")?;
                }
                None => {
                    write_error(&mut handle, "3 no prompt")?;
                }
            },
            Command::BYE => {
                break;
            }
            Command::SETERROR(_e) => {
                Notification::new()
                    .summary("rbw - Error")
                    .body(
                        "Master password is incorrect. Use 'rbw-pinentry store' to edit the entry.",
                    )
                    .show()?;
                write_error(&mut handle, "4 incorrect master password")?;
            }
            _ => {
                write_error(&mut handle, "5 unknown command")?;
            }
        }
    }

    Ok(())
}
