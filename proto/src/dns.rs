use crate::helper::*;
use crate::qclass::{as_qclass, as_u16 as qclass_as_u16, QClass};
use crate::qtype::{as_qtype, as_u16 as qtype_as_u16, QType};
use crate::reader::Reader;
use crate::writer::Writer;

use failure::Error;

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
    rclass: QClass,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

impl DNS {
    pub fn new(byte_arr: Vec<u8>) -> Result<Self, Error> {
        let mut reader = Reader::new(&byte_arr);
        let id = reader.read_u16_be()?;

        let parsed_flags = reader.read_u8_as_binary()?;
        let opcode_bin = [
            parsed_flags[1],
            parsed_flags[2],
            parsed_flags[3],
            parsed_flags[4],
        ];
        let qr = parsed_flags[0];
        let opcode = four_bit_to_u8(opcode_bin);
        let aa = parsed_flags[5];
        let tc = parsed_flags[6];
        let rd = parsed_flags[7];

        let parsed_flags2 = reader.read_u8_as_binary()?;
        let rcode_bin = [
            parsed_flags2[4],
            parsed_flags2[5],
            parsed_flags2[6],
            parsed_flags2[7],
        ];
        let ra = parsed_flags2[0];
        let z = 0;
        let rcode = four_bit_to_u8(rcode_bin);

        let qdcount = reader.read_u16_be()?;
        let ancount = reader.read_u16_be()?;
        let nscount = reader.read_u16_be()?;
        let arcount = reader.read_u16_be()?;

        let mut questions = Vec::new();

        let mut question_length = reader.read_u8()?;
        let mut qname = Vec::new();

        while question_length != 0 {
            let mut name = reader.read_length(question_length as usize)?;
            qname.append(&mut name);
            question_length = reader.read_u8()?;

            if question_length != 0 {
                qname.push(46);
            }
        }

        let qname = String::from_utf8(qname)?;
        let qtype = as_qtype(reader.read_u16_be()?);
        let qclass = as_qclass(reader.read_u16_be()?);

        questions.push(Question {
            qname,
            qtype,
            qclass,
        });

        let mut resource_records = Vec::new();
        if qr == 1 {
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

                let rtype = as_qtype(reader.read_u16_be()?);
                let rclass = as_qclass(reader.read_u16_be()?);

                let ttl = reader.read_u32_be()?;
                let rdlength = reader.read_u16_be()?;
                let rdata = reader.read_length(rdlength as usize)?;

                let resource_record = ResourceRecord {
                    name: String::from_utf8(name)?,
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
        let opcode = u8_to_four_bit(self.opcode);
        let flags = [
            self.qr, opcode[0], opcode[1], opcode[2], opcode[3], self.aa, self.tc, self.rd,
        ];

        let rcode = u8_to_four_bit(self.rcode);
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

        if self.ancount >= 1 {
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
    pub fn test_parse_response_google() {
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
    pub fn test_parse_response_github() {
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
}
