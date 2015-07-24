use std::convert::From;
use std::error;
use std::fmt;
use std::io;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Format(String),
    Generic
}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_str("ParseError")
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Io(_) => "I/O error",
            ParseError::Format(ref msg) => msg,
            ParseError::Generic => "unknown error"
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ParseError::Io(ref e) => Some(e as &error::Error),
            _ => None
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::Io(err)
    }
}
