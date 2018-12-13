#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
