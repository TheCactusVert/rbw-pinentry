use std::{io::Write, str::FromStr};

use percent_encoding::{CONTROLS, percent_encode};
use regex::Regex;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pinentry {
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

pub fn write_introduction<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(
        out,
        "OK Pleased to meet you, process {}",
        std::process::id()
    )
}

pub fn write_ok<T: Write>(out: &mut T) -> std::io::Result<()> {
    writeln!(out, "OK")
}

pub fn write_password<T: Write>(out: &mut T, password: &str) -> std::io::Result<()> {
    let password = percent_encode(password.as_bytes(), CONTROLS);
    writeln!(out, "D {password}")
}

pub fn write_error<T: Write>(out: &mut T, message: &str) -> std::io::Result<()> {
    writeln!(out, "ERR {message}")
}
