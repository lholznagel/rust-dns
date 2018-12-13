mod helper;
mod parser;
mod qclass;
mod qtype;

use crate::helper::*;
use crate::parser::Parser;
use crate::qclass::{as_qclass, QClass};
use crate::qtype::{as_qtype, QType};

use failure::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DNS {
    pub id: u16,
    pub qr: u8,
    pub opcode: u8,
    pub aa: u8,
    pub tc: u8,
    pub rd: u8,
    pub ra: u8,
    pub z: u8,
    pub rcode: u8,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
    pub questions: Vec<Question>,
    pub resource_records: Vec<ResourceRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Question {
    pub qname: String,
    pub qtype: QType,
    pub qclass: QClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceRecord {
    name: String,
    rtype: QType,
    class: QClass,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

impl DNS {
    pub fn new(byte_arr: Vec<u8>) -> Result<Self, Error> {
        let mut parser = Parser::new(&byte_arr);
        let id = parser.read_u16_be()?;

        let parsed_flags = parser.read_u8()?;
        let parsed_flags_binary = to_binary(parsed_flags);

        let opcode_bin = [
            parsed_flags_binary[1],
            parsed_flags_binary[2],
            parsed_flags_binary[3],
            parsed_flags_binary[4],
        ];
        let qr = parsed_flags_binary[0];
        let opcode = four_bit_to_u8(opcode_bin);
        let aa = parsed_flags_binary[5];
        let tc = parsed_flags_binary[6];
        let rd = parsed_flags_binary[7];

        let parsed_flags2 = parser.read_u8()?;
        let parsed_flags2_binary = to_binary(parsed_flags2);

        let rcode_bin = [
            parsed_flags2_binary[4],
            parsed_flags2_binary[5],
            parsed_flags2_binary[6],
            parsed_flags2_binary[7],
        ];
        let ra = parsed_flags2_binary[0];
        let z = 0;
        let rcode = four_bit_to_u8(rcode_bin);

        let qdcount = parser.read_u16_be()?;
        let ancount = parser.read_u16_be()?;
        let nscount = parser.read_u16_be()?;
        let arcount = parser.read_u16_be()?;

        let mut questions = Vec::new();

        let mut question_length = parser.read_u8()?;
        let mut qname = Vec::new();

        while question_length != 0 {
            let mut name = parser.read_length(question_length as usize)?;
            qname.append(&mut name);
            question_length = parser.read_u8()?;

            if question_length != 0 {
                qname.push(46);
            }
        }

        let qname = String::from_utf8(qname)?;
        let qtype = as_qtype(parser.read_u16_be()?);
        let qclass = as_qclass(parser.read_u16_be()?);

        questions.push(Question {
            qname,
            qtype,
            qclass,
        });

        let mut resource_records = Vec::new();
        if qr == 1 {
            for _ in 0..ancount {
                let mut name = Vec::new();
                let name_offset = parser.read_u8()?;
                let mut currrent_position = 0;

                if name_offset == 192 {
                    let offset = parser.read_u8()?;
                    currrent_position = parser.position();
                    parser.set_position(u64::from(offset));

                    let mut next_length = parser.read_u8()?;
                    while next_length != 0 {
                        let mut rname = parser.read_length(next_length as usize)?;
                        name.append(&mut rname);
                        next_length = parser.read_u8()?;

                        if next_length != 0 {
                            name.push(46);
                        }
                    }
                }

                parser.set_position(currrent_position);

                let rtype = as_qtype(parser.read_u16_be()?);
                let rclass = as_qclass(parser.read_u16_be()?);

                let ttl = parser.read_u32_be()?;
                let rdlength = parser.read_u16_be()?;
                let rdata = parser.read_length(rdlength as usize)?;

                let resource_record = ResourceRecord {
                    name: String::from_utf8(name)?,
                    rtype,
                    class: rclass,
                    ttl,
                    rdlength,
                    rdata,
                };

                resource_records.push(resource_record);
            }
        }

        Ok(Self {
            id,
            qr,
            opcode,
            aa,
            tc,
            rd,
            ra,
            z,
            rcode,
            qdcount,
            ancount,
            nscount,
            arcount,
            questions,
            resource_records,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    pub fn test_parser_question_google() {
        let hex_query = "349e010000010000000000000377777706676f6f676c650264650000010001";
        let hex_query = hex::decode(hex_query).unwrap();
        let dns = DNS::new(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 13470,
                qr: 0,
                opcode: 0,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 0,
                z: 0,
                rcode: 0,
                qdcount: 1,
                ancount: 0,
                nscount: 0,
                arcount: 0,
                questions: vec![Question {
                    qname: String::from("www.google.de"),
                    qtype: QType::A,
                    qclass: QClass::IN
                }],
                resource_records: Vec::new()
            }
        );
    }

    #[test]
    pub fn test_parser_response_google() {
        let hex_response = "349e818000010001000000000377777706676f6f676c650264650000010001c00c00010001000000ee0004acd9a8c3";
        let hex_query = hex::decode(hex_response).unwrap();
        let dns = DNS::new(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 13470,
                qr: 1,
                opcode: 0,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 1,
                z: 0,
                rcode: 0,
                qdcount: 1,
                ancount: 1,
                nscount: 0,
                arcount: 0,
                questions: vec![Question {
                    qname: String::from("www.google.de"),
                    qtype: QType::A,
                    qclass: QClass::IN
                }],
                resource_records: vec![ResourceRecord {
                    name: String::from("www.google.de"),
                    rtype: QType::A,
                    class: QClass::IN,
                    ttl: 238,
                    rdlength: 4,
                    rdata: vec![172, 217, 168, 195]
                }]
            }
        );
    }

    #[test]
    pub fn test_parser_question_github() {
        let hex_query = "224c01000001000000000000037777770667697468756203636f6d0000010001";
        let hex_query = hex::decode(hex_query).unwrap();
        let dns = DNS::new(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 8780,
                qr: 0,
                opcode: 0,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 0,
                z: 0,
                rcode: 0,
                qdcount: 1,
                ancount: 0,
                nscount: 0,
                arcount: 0,
                questions: vec![Question {
                    qname: String::from("www.github.com"),
                    qtype: QType::A,
                    qclass: QClass::IN
                }],
                resource_records: Vec::new()
            }
        );
    }

    #[test]
    pub fn test_parser_response_github() {
        let hex_response = "224c81800001000300000000037777770667697468756203636f6d0000010001c00c00050001000004930002c010c010000100010000003b0004c01efd71c010000100010000003b0004c01efd70";
        let hex_query = hex::decode(hex_response).unwrap();
        let dns = DNS::new(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 8780,
                qr: 1,
                opcode: 0,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 1,
                z: 0,
                rcode: 0,
                qdcount: 1,
                ancount: 3,
                nscount: 0,
                arcount: 0,
                questions: vec![Question {
                    qname: String::from("www.github.com"),
                    qtype: QType::A,
                    qclass: QClass::IN
                }],
                resource_records: vec![
                    ResourceRecord {
                        name: String::from("www.github.com"),
                        rtype: QType::CNAME,
                        class: QClass::IN,
                        ttl: 1171,
                        rdlength: 2,
                        rdata: vec![192, 16]
                    },
                    ResourceRecord {
                        name: String::from("github.com"),
                        rtype: QType::A,
                        class: QClass::IN,
                        ttl: 59,
                        rdlength: 4,
                        rdata: vec![192, 30, 253, 113]
                    },
                    ResourceRecord {
                        name: String::from("github.com"),
                        rtype: QType::A,
                        class: QClass::IN,
                        ttl: 59,
                        rdlength: 4,
                        rdata: vec![192, 30, 253, 112]
                    }
                ]
            }
        );
    }
}
