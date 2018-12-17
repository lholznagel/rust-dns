#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum QType {
    A,
    NS,
    MD,
    MF,
    CNAME,
    SOA,
    MB,
    MG,
    MR,
    NULL,
    WKS,
    PTR,
    HINFO,
    MINFO,
    MX,
    TXT,
    AAAA,
}

pub fn as_qtype(val: u16) -> QType {
    match val {
        1 => QType::A,
        2 => QType::NS,
        3 => QType::MD,
        4 => QType::MF,
        5 => QType::CNAME,
        6 => QType::SOA,
        7 => QType::MB,
        8 => QType::MG,
        9 => QType::MR,
        10 => QType::NULL,
        11 => QType::WKS,
        12 => QType::PTR,
        13 => QType::HINFO,
        14 => QType::MINFO,
        15 => QType::MX,
        16 => QType::TXT,
        28 => QType::AAAA,
        // TODO: throw error
        _ => panic!("Unknown qtype"),
    }
}

pub fn as_u16(val: QType) -> u16 {
    match val {
        QType::A => 1,
        QType::NS => 2,
        QType::MD => 3,
        QType::MF => 4,
        QType::CNAME => 5,
        QType::SOA => 6,
        QType::MB => 7,
        QType::MG => 8,
        QType::MR => 9,
        QType::NULL => 10,
        QType::WKS => 11,
        QType::PTR => 12,
        QType::HINFO => 13,
        QType::MINFO => 14,
        QType::MX => 15,
        QType::TXT => 16,
        QType::AAAA => 28,
    }
}
