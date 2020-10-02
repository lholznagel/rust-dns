mod opcode;
mod rcode;

pub use self::opcode::*;
pub use self::rcode::*;

use crate::error::*;
use crate::qclass::{as_u16 as qclass_as_u16, QClass};
use crate::qtype::{as_u16 as qtype_as_u16, QType};
use crate::reader::*;
use crate::writer::Writer;

use std::io::Cursor;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Question {
    pub qname: String,
    pub qtype: QType,
    pub qclass: QClass,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ResourceRecord {
    pub name: String,
    pub rtype: QType,
    pub rclass: QClass,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct DNS {
    pub id: u16,
    pub qr: u8,
    pub opcode: Opcode,
    pub aa: u8,
    pub tc: u8,
    pub rd: u8,
    pub ra: u8,
    pub z: u8,
    pub rcode: Rcode,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16, // currently ignored
    pub arcount: u16, // currently ignored
    pub questions: Vec<Question>,
    pub resource_records: Vec<ResourceRecord>,
}

impl DNS {
    pub fn parse(byte_arr: Vec<u8>) -> Result<Self> {
        let mut protocol = Cursor::new(byte_arr.clone());

        let mut reader = Reader::new(&byte_arr);
        let id = protocol.read_u16()?;

        let flags = protocol.read_binary()?;
        let qr = flags[0];
        let opcode = Opcode::from(&flags[1..=4]);
        let aa = flags[5];
        let tc = flags[6];
        let rd = flags[7];

        let flags = protocol.read_binary()?;
        let ra = flags[0];
        let rcode = Rcode::from(&flags[4..=7]);
        let z = 0;

        let qdcount = protocol.read_u16()?;
        let ancount = protocol.read_u16()?;
        let nscount = protocol.read_u16()?;
        let arcount = protocol.read_u16()?;

        let mut questions = Vec::with_capacity(1);
        for _ in 0..qdcount {
            let mut domain = Vec::with_capacity(3);
            let mut qname_length = protocol.read_u8()?;

            while qname_length != 0 {
                let qname = protocol.read_length(qname_length as usize)?;
                domain.push(qname);
                qname_length = protocol.read_u8()?;
            }

            let qname = String::from_utf8(domain.join(&46)).map_err(DnsParseError::StringParseError)?;
            let qtype = QType::from(protocol.read_u16()?);
            let qclass = QClass::from(protocol.read_u16()?);

            questions.push(Question {
                qname,
                qtype,
                qclass,
            });
        }

        let mut resource_records = Vec::new();
        if qr == 1 {
            dbg!(protocol);
            for _ in 0..ancount {
                let mut name = Vec::new();
                let name_offset = reader.read_u8()?;
                let mut currrent_position = 0;

                if name_offset == 192 {
                    let offset = reader.read_u8()?;
                    currrent_position = reader.position();
                    reader.set_position(u64::from(offset));

                    let mut next_length = reader.read_u8()?;
                    while next_length != 0 {
                        let mut rname = reader.read_length(next_length as usize)?;
                        name.append(&mut rname);
                        next_length = reader.read_u8()?;

                        if next_length != 0 {
                            name.push(46);
                        }
                    }
                }

                reader.set_position(currrent_position);

                let rtype = crate::qtype::as_qtype(reader.read_u16_be()?);
                let rclass = crate::qclass::as_qclass(reader.read_u16_be()?);

                let ttl = reader.read_u32_be()?;
                let rdlength = reader.read_u16_be()?;
                let rdata = reader.read_length(rdlength as usize)?;

                let resource_record = ResourceRecord {
                    name: String::from_utf8(name).map_err(DnsParseError::StringParseError)?,
                    rtype,
                    rclass,
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

    pub fn build(self) -> Vec<u8> {
        let opcode: &[u8] = self.opcode.into();
        //let opcode = u8_to_four_bit(self.opcode);
        let flags = [
            self.qr, opcode[0], opcode[1], opcode[2], opcode[3], self.aa, self.tc, self.rd,
        ];

        let rcode: &[u8] = self.rcode.into();
        let flags2 = [self.ra, 0, 0, 0, rcode[0], rcode[1], rcode[2], rcode[3]];

        let writer = Writer::with_capacity(128)
            .write_u16_be(self.id)
            .write_binary_as_u8(flags)
            .write_binary_as_u8(flags2)
            .write_u16_be(self.questions.len() as u16)
            .write_u16_be(self.resource_records.len() as u16)
            .write_u16_be(0) // nscount
            .write_u16_be(0); // arcount

        let mut questions = Vec::new();
        for question in self.questions {
            let splitted_qname = question.qname.split('.').collect::<Vec<&str>>();

            for name in splitted_qname {
                questions.push(name.len() as u8);
                questions.append(&mut name.as_bytes().to_vec());
            }

            let mut writer = Writer::new()
                .write_u8(0) // name is done
                .write_u16_be(qtype_as_u16(question.qtype))
                .write_u16_be(qclass_as_u16(question.qclass))
                .build();
            questions.append(&mut writer);
        }

        let mut position_question = writer.position();
        let writer = writer.write_vec(questions);

        if !self.resource_records.is_empty() {
            let mut responses = Vec::new();
            for resource in self.resource_records {
                let mut writer = Writer::new()
                    .write_u8(192)
                    .write_u8(position_question as u8)
                    .write_u16_be(qtype_as_u16(resource.rtype))
                    .write_u16_be(qclass_as_u16(resource.rclass))
                    .write_u32_be(resource.ttl)
                    .write_u16_be(resource.rdlength)
                    .write_vec(resource.rdata)
                    .build();
                responses.append(&mut writer);

                if resource.rtype == QType::CNAME {
                    position_question += 4; // ignore www.
                }
            }

            writer.write_vec(responses).build()
        } else {
            writer.build()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    pub fn test_parse_query_google() {
        let hex_query = "349e010000010000000000000377777706676f6f676c650264650000010001";
        let hex_query = hex::decode(hex_query).unwrap();
        let dns = DNS::parse(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 13470,
                qr: 0,
                opcode: Opcode::Query,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 0,
                z: 0,
                rcode: Rcode::NoError,
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
    pub fn test_parse_response_google() {
        let hex_response = "349e818000010001000000000377777706676f6f676c650264650000010001c00c00010001000000ee0004acd9a8c3";
        let hex_query = hex::decode(hex_response).unwrap();
        let dns = DNS::parse(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 13470,
                qr: 1,
                opcode: Opcode::Query,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 1,
                z: 0,
                rcode: Rcode::NoError,
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
                    rclass: QClass::IN,
                    ttl: 238,
                    rdlength: 4,
                    rdata: vec![172, 217, 168, 195]
                }]
            }
        );
    }

    #[test]
    pub fn test_build_query_google() {
        let vector = DNS {
            id: 13470,
            qr: 0,
            opcode: Opcode::Query,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 0,
            z: 0,
            rcode: Rcode::NoError,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.google.de"),
                qtype: QType::A,
                qclass: QClass::IN,
            }],
            resource_records: Vec::new(),
        }
        .build();

        let hex_query = "349e010000010000000000000377777706676f6f676c650264650000010001";
        let hex_vec = hex::encode(vector);
        assert_eq!(hex_vec, hex_query);
    }

    #[test]
    pub fn test_build_response_google() {
        let vector = DNS {
            id: 13470,
            qr: 1,
            opcode: Opcode::Query,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 1,
            z: 0,
            rcode: Rcode::NoError,
            qdcount: 1,
            ancount: 1,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.google.de"),
                qtype: QType::A,
                qclass: QClass::IN,
            }],
            resource_records: vec![ResourceRecord {
                name: String::from("www.google.de"),
                rtype: QType::A,
                rclass: QClass::IN,
                ttl: 238,
                rdlength: 4,
                rdata: vec![172, 217, 168, 195],
            }],
        }
        .build();

        let hex_query = "349e818000010001000000000377777706676f6f676c650264650000010001c00c00010001000000ee0004acd9a8c3";
        let hex_vec = hex::encode(vector);
        assert_eq!(hex_vec, hex_query);
    }

    #[test]
    pub fn test_parse_query_github() {
        let hex_query = "224c01000001000000000000037777770667697468756203636f6d0000010001";
        let hex_query = hex::decode(hex_query).unwrap();
        let dns = DNS::parse(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 8780,
                qr: 0,
                opcode: Opcode::Query,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 0,
                z: 0,
                rcode: Rcode::NoError,
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
    pub fn test_parse_response_github() {
        let hex_response = "224c81800001000300000000037777770667697468756203636f6d0000010001c00c00050001000004930002c010c010000100010000003b0004c01efd71c010000100010000003b0004c01efd70";
        let hex_query = hex::decode(hex_response).unwrap();
        let dns = DNS::parse(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 8780,
                qr: 1,
                opcode: Opcode::Query,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 1,
                z: 0,
                rcode: Rcode::NoError,
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
                        rclass: QClass::IN,
                        ttl: 1171,
                        rdlength: 2,
                        rdata: vec![192, 16]
                    },
                    ResourceRecord {
                        name: String::from("github.com"),
                        rtype: QType::A,
                        rclass: QClass::IN,
                        ttl: 59,
                        rdlength: 4,
                        rdata: vec![192, 30, 253, 113]
                    },
                    ResourceRecord {
                        name: String::from("github.com"),
                        rtype: QType::A,
                        rclass: QClass::IN,
                        ttl: 59,
                        rdlength: 4,
                        rdata: vec![192, 30, 253, 112]
                    }
                ]
            }
        );
    }

    #[test]
    pub fn test_build_query_github() {
        let vector = DNS {
            id: 8780,
            qr: 0,
            opcode: Opcode::Query,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 0,
            z: 0,
            rcode: Rcode::NoError,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.github.com"),
                qtype: QType::A,
                qclass: QClass::IN,
            }],
            resource_records: Vec::new(),
        }
        .build();

        let hex_query = "224c01000001000000000000037777770667697468756203636f6d0000010001";
        let hex_vec = hex::encode(vector);
        assert_eq!(hex_vec, hex_query);
    }

    #[test]
    pub fn test_build_response_github() {
        let vector = DNS {
            id: 8780,
            qr: 1,
            opcode: Opcode::Query,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 1,
            z: 0,
            rcode: Rcode::NoError,
            qdcount: 1,
            ancount: 3,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.github.com"),
                qtype: QType::A,
                qclass: QClass::IN,
            }],
            resource_records: vec![
                ResourceRecord {
                    name: String::from("www.github.com"),
                    rtype: QType::CNAME,
                    rclass: QClass::IN,
                    ttl: 1171,
                    rdlength: 2,
                    rdata: vec![192, 16],
                },
                ResourceRecord {
                    name: String::from("github.com"),
                    rtype: QType::A,
                    rclass: QClass::IN,
                    ttl: 59,
                    rdlength: 4,
                    rdata: vec![192, 30, 253, 113],
                },
                ResourceRecord {
                    name: String::from("github.com"),
                    rtype: QType::A,
                    rclass: QClass::IN,
                    ttl: 59,
                    rdlength: 4,
                    rdata: vec![192, 30, 253, 112],
                },
            ],
        }
        .build();

        let hex_query = "224c81800001000300000000037777770667697468756203636f6d0000010001c00c00050001000004930002c010c010000100010000003b0004c01efd71c010000100010000003b0004c01efd70";
        let hex_vec = hex::encode(vector);
        assert_eq!(hex_vec, hex_query);
    }

    #[test]
    pub fn test_parse_query_play_google_aaaa() {
        let hex_query = "8af00100000100000000000004706c617906676f6f676c6503636f6d00001c0001";
        let hex_query = hex::decode(hex_query).unwrap();
        let dns = DNS::parse(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 35568,
                qr: 0,
                opcode: Opcode::Query,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 0,
                z: 0,
                rcode: Rcode::NoError,
                qdcount: 1,
                ancount: 0,
                nscount: 0,
                arcount: 0,
                questions: vec![Question {
                    qname: String::from("play.google.com"),
                    qtype: QType::AAAA,
                    qclass: QClass::IN
                }],
                resource_records: Vec::new()
            }
        );
    }

    #[test]
    pub fn test_parse_response_play_google_aaaa() {
        let hex_query = "8af08180000100010000000004706c617906676f6f676c6503636f6d00001c0001c00c001c00010000006c00102a00145040010815000000000000200e";
        let hex_query = hex::decode(hex_query).unwrap();
        let dns = DNS::parse(hex_query).unwrap();

        assert_eq!(
            dns,
            DNS {
                id: 35568,
                qr: 1,
                opcode: Opcode::Query,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 1,
                z: 0,
                rcode: Rcode::NoError,
                qdcount: 1,
                ancount: 1,
                nscount: 0,
                arcount: 0,
                questions: vec![Question {
                    qname: String::from("play.google.com"),
                    qtype: QType::AAAA,
                    qclass: QClass::IN
                }],
                resource_records: vec![ResourceRecord {
                    name: String::from("play.google.com"),
                    rtype: QType::AAAA,
                    rclass: QClass::IN,
                    ttl: 108,
                    rdlength: 16,
                    rdata: vec![42, 0, 20, 80, 64, 1, 8, 21, 0, 0, 0, 0, 0, 0, 32, 14]
                }]
            }
        );
    }

    #[test]
    pub fn test_build_query_play_google_aaaa() {
        let vector = DNS {
            id: 35568,
            qr: 0,
            opcode: Opcode::Query,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 0,
            z: 0,
            rcode: Rcode::NoError,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("play.google.com"),
                qtype: QType::AAAA,
                qclass: QClass::IN,
            }],
            resource_records: Vec::new(),
        }
        .build();

        let hex_query = "8af00100000100000000000004706c617906676f6f676c6503636f6d00001c0001";
        let hex_vec = hex::encode(vector);
        assert_eq!(hex_vec, hex_query);
    }

    #[test]
    pub fn test_build_response_play_google_aaaa() {
        let vector = DNS {
            id: 35568,
            qr: 1,
            opcode: Opcode::Query,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 1,
            z: 0,
            rcode: Rcode::NoError,
            qdcount: 1,
            ancount: 1,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("play.google.com"),
                qtype: QType::AAAA,
                qclass: QClass::IN,
            }],
            resource_records: vec![ResourceRecord {
                name: String::from("play.google.com"),
                rtype: QType::AAAA,
                rclass: QClass::IN,
                ttl: 108,
                rdlength: 16,
                rdata: vec![42, 0, 20, 80, 64, 1, 8, 21, 0, 0, 0, 0, 0, 0, 32, 14],
            }],
        }
        .build();

        let hex_query = "8af08180000100010000000004706c617906676f6f676c6503636f6d00001c0001c00c001c00010000006c00102a00145040010815000000000000200e";
        let hex_vec = hex::encode(vector);
        assert_eq!(hex_vec, hex_query);
    }
}
