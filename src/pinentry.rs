use std::io::{self, BufRead, Write};

use anyhow::Result;
use keyring_core::Entry;
use lazy_regex::regex_captures;
use notify_rust::Notification;
use percent_encoding::{CONTROLS, percent_encode};

enum Command<'a> {
    SETTITLE(&'a str),
    SETDESC(&'a str),
    SETPROMPT(&'a str),
    SETERROR(&'a str),
    GETPIN,
    BYE,
    UNKNOWN,
}

impl<'a> From<&'a str> for Command<'a> {
    fn from(input: &'a str) -> Self {
        match regex_captures!(r"^(\w*) ?(.*)?$", input) {
            Some((_, command, arg)) => match command.to_ascii_uppercase().as_str() {
                "SETTITLE" => Self::SETTITLE(arg),
                "SETDESC" => Self::SETDESC(arg),
                "SETPROMPT" => Self::SETPROMPT(arg),
                "SETERROR" => Self::SETERROR(arg),
                "GETPIN" => Self::GETPIN,
                "BYE" => Self::BYE,
                _ => Self::UNKNOWN,
            },
            None => Self::UNKNOWN,
        }
    }
}

fn write_greet<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(
        out,
        "OK Pleased to meet you, process {}",
        std::process::id()
    )
}

fn write_ok<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(out, "OK")
}

fn write_bye<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(out, "OK closing connection")
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

    write_greet(&mut handle)?;

    for line in stdin.lock().lines() {
        match Command::from(line.unwrap().as_str()) {
            Command::SETTITLE(_arg) => {
                write_ok(&mut handle)?;
            }
            Command::SETDESC(_arg) => {
                write_ok(&mut handle)?;
            }
            Command::SETPROMPT(arg) => {
                prompt = Some(arg.to_string());
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
                            write_error(&mut handle, "1 No master password")?;
                        }
                    };
                }
                Some(_) => {
                    write_error(&mut handle, "2 Unknown prompt")?;
                }
                None => {
                    write_error(&mut handle, "3 No prompt")?;
                }
            },
            Command::BYE => {
                write_bye(&mut handle)?;
                break;
            }
            Command::SETERROR(_e) => {
                Notification::new()
                    .summary("rbw - Error")
                    .body(
                        "Master password is incorrect. Use 'rbw-pinentry store' to edit the entry.",
                    )
                    .show()?;
                write_error(&mut handle, "4 Incorrect master password")?;
            }
            _ => {
                write_error(&mut handle, "5 Unknown command")?;
            }
        }
    }

    Ok(())
}
