use std::fmt::Display;
use std::io;
use std::num;

use owo_colors::OwoColorize;
use regex::Regex;

use super::cli::Cli;

#[macro_export]
macro_rules! fatal {
    ($($tt:tt)*) => {
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($tt)*).unwrap();
        ::std::process::exit(1);
    }
}

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Parse(num::ParseIntError),
    Regex(regex::Error),
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<num::ParseIntError> for CliError {
    fn from(err: num::ParseIntError) -> Self {
        CliError::Parse(err)
    }
}

impl From<regex::Error> for CliError {
    fn from(err: regex::Error) -> Self {
        CliError::Regex(err)
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Parse(ref err) => err.fmt(f),
            CliError::Regex(ref err) => err.fmt(f),
        }
    }
}
