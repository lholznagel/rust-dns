#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Opcode {
    Query,
    IQuery,
    Status,
    Reserved,
}

impl Default for Opcode {
    fn default() -> Self {
        Self::Query
    }
}

impl From<&[u8]> for Opcode {
    /// Will silently declare everything invalid as reserved
    fn from(x: &[u8]) -> Self {
        match x {
            [0, 0, 0, 0] => Self::Query,
            [0, 0, 0, 1] => Self::IQuery,
            [0, 0, 1, 0] => Self::Status,
            _ => Self::Reserved,
        }
    }
}

impl Into<&[u8]> for Opcode {
    fn into(self) -> &'static [u8] {
        match self {
            Self::Query => &[0, 0, 0, 0],
            Self::IQuery => &[0, 0, 0, 1],
            Self::Status => &[0, 0, 1, 0],
            _ => &[1, 1, 1, 1],
        }
    }
}