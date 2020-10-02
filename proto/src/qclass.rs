#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum QClass {
    IN,
}

impl From<u16> for QClass {
    fn from(x: u16) -> Self {
        match x {
            1 => QClass::IN,
            _ => panic!("Unknown qtype"),
        }
    }
}

pub fn as_qclass(val: u16) -> QClass {
    match val {
        1 => QClass::IN,
        _ => panic!("Unknown qtype"),
    }
}

pub fn as_u16(val: QClass) -> u16 {
    match val {
        QClass::IN => 1,
    }
}
