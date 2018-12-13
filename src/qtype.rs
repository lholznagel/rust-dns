#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
        // TODO: throw error
        _ => panic!("Unknown qtype"),
    }
}
