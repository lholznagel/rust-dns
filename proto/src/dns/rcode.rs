#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Rcode {
    /// 0 -> No error condition
    NoError,
    /// 1 -> The name server was unable to interpret the query
    FormatError,
    /// 2 -> The name server was unable to process this query due to a problem with the name server
    ServerFailure,
    /// 3 -> Meaningful only for responses from an authoritative name server, this code signifies that the domain name referenced in the query does not exist
    NameError,
    /// 4 -> The name server does not support the requested kind of query
    NotImplemented,
    /// 5 -> The name server refuses to perform the specified operation for policy reasons.
    /// For example, a name server may not wish to provide the information to the particular requester, or 
    /// a name server may not wish to perform a particular operation (e.g., zone transfer) for particular data
    Refused,
    Reserved,
}

impl Default for Rcode {
    fn default() -> Self {
        Self::NoError
    }
}

impl From<&[u8]> for Rcode {
    /// Will silently declare everything invalid as reserved
    fn from(x: &[u8]) -> Self {
        match x {
            [0, 0, 0, 0] => Self::NoError,
            [0, 0, 0, 1] => Self::FormatError,
            [0, 0, 1, 0] => Self::ServerFailure,
            [0, 0, 1, 1] => Self::NameError,
            [0, 1, 0, 0] => Self::NotImplemented,
            [0, 1, 0, 1] => Self::Refused,
            _ => Self::Reserved,
        }
    }
}

impl Into<&[u8]> for Rcode {
    fn into(self) -> &'static [u8] {
        match self {
            Self::NoError => &[0, 0, 0, 0],
            Self::FormatError => &[0, 0, 0, 1],
            Self::ServerFailure => &[0, 0, 1, 0],
            Self::NameError => &[0, 0, 1, 1],
            Self::NotImplemented => &[0, 1, 0, 0],
            Self::Refused => &[0, 1, 0, 1],
            _ => &[1, 1, 1, 1],
        }
    }
}