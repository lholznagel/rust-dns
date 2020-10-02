use std::error::Error;
use std::fmt;

pub(crate) type Result<T> = std::result::Result<T, RdnsError>;

#[derive(Debug)]
pub enum RdnsError {
    IoError(std::io::Error),
    Todo,
}

impl Error for RdnsError {}

impl fmt::Display for RdnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for RdnsError {
    fn from(x: std::io::Error) -> Self {
        RdnsError::IoError(x)
    }
}
