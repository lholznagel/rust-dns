#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum QClass {
    IN,
}

pub fn as_qclass(val: u16) -> QClass {
    match val {
        1 => QClass::IN,
        // TODO: throw error
        _ => panic!("Unknown qtype"),
    }
}

pub fn as_u16(val: QClass) -> u16 {
    match val {
        QClass::IN => 1,
    }
}
