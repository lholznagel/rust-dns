use std::error::Error;
use std::fmt;

pub(crate) type Result<T> = std::result::Result<T, DnsParseError>;

#[derive(Debug)]
pub enum DnsParseError {
    IoError(std::io::Error),
    StringParseError(std::string::FromUtf8Error),
}

impl Error for DnsParseError {}

impl fmt::Display for DnsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for DnsParseError {
    fn from(x: std::io::Error) -> Self {
        DnsParseError::IoError(x)
    }
}
