mod parser;

use failure::Error;
use crate::parser::Parser;

fn main() {
    println!("Hello, world!");
}

fn to_binary(val: u8) -> [u8; 8] {
    let mut val = val;
    let mut result = [0; 8];

    if val >= 128 {
        result[0] = 1;
        val = val - 128;
    }

    if val >= 64 {
        result[1] = 1;
        val = val - 64;
    }

    if val >= 32 {
        result[2] = 1;
        val = val - 32;
    }

    if val >= 16 {
        result[3] = 1;
        val = val - 16;
    }

    if val >= 8 {
        result[4] = 1;
        val = val - 8;
    }

    if val >= 4 {
        result[5] = 1;
        val = val - 4;
    }

    if val >= 2 {
        result[6] = 1;
        val = val - 2;
    }

    if val >= 1 {
        result[7] = 1;
    }

    result
}

pub fn four_bit_to_u8(val: [u8; 4]) -> u8 {
    let mut result = 0;

    if val[0] == 1 {
        result += 8;
    }

    if val[1] == 1 {
        result += 4;
    }

    if val[2] == 1 {
        result += 2;
    }

    if val[3] == 1 {
        result += 1;
    }

    result
}

#[derive(Debug, Eq, PartialEq)]
struct DNS {
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
    pub resource_records: Vec<ResourceRecord>
}

#[derive(Debug, Eq, PartialEq)]
struct Question {
    pub qname: String,
    pub qtype: String,
    pub qclass: String
}

#[derive(Debug, Eq, PartialEq)]
struct ResourceRecord {
    name: String,
    rtype: String,
    class: String,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>
}

impl DNS {
    pub fn new(byte_arr: Vec<u8>) -> Result<Self, Error> {
        let mut parser = Parser::new(&byte_arr);
        let id = parser.read_u16()?.to_be();

        let parsed_flags = parser.read_u8()?;
        let parsed_flags_binary = to_binary(parsed_flags);

        let opcode_bin = [parsed_flags_binary[1], parsed_flags_binary[2], parsed_flags_binary[3], parsed_flags_binary[4]];
        let qr = parsed_flags_binary[0];
        let opcode = four_bit_to_u8(opcode_bin);
        let aa = parsed_flags_binary[5];
        let tc = parsed_flags_binary[6];
        let rd = parsed_flags_binary[7];

        let parsed_flags2 = parser.read_u8()?;
        let parsed_flags2_binary = to_binary(parsed_flags2);

        let rcode_bin = [parsed_flags2_binary[4], parsed_flags2_binary[5], parsed_flags2_binary[6], parsed_flags2_binary[7]];
        let ra = parsed_flags2_binary[0];
        let z = 0;
        let rcode = four_bit_to_u8(rcode_bin);

        let qdcount = parser.read_u16()?.to_be();
        let ancount = parser.read_u16()?.to_be();
        let nscount = parser.read_u16()?.to_be();
        let arcount = parser.read_u16()?.to_be();

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

        let _ = parser.read_u16()?; // qtype
        let _ = parser.read_u16()?; // qclass
        let _ = parser.read_u16()?; // ?
        let _ = parser.read_u16()?; // ?
        let _ = parser.read_u16()?; // ?

        let qname = String::from_utf8(qname)?;
        let qtype = String::from("A"); // TODO write parser
        let qclass = String::from("IN"); // TODO write parser

        questions.push(Question {
            qname: qname.clone(),
            qtype: qtype.clone(),
            qclass: qclass.clone()
        });

        let mut resource_records = Vec::new();
        if qr == 1 {
            let ttl = parser.read_u32_be()?;
            let rdlength = parser.read_u16_be()?;
            let rdata = parser.read_length(rdlength as usize)?;

            let resource_record = ResourceRecord {
                name: qname,
                rtype: qtype,
                class: qclass,
                ttl,
                rdlength,
                rdata
            };

            resource_records.push(resource_record);
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
            resource_records
        })
    }
}

#[cfg(test)]
mod tests {
    use hex;
    use super::*;

    #[test]
    pub fn test_to_binary() {
        assert_eq!(to_binary(255), [1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(to_binary(128), [1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(to_binary(65), [0, 1, 0, 0, 0, 0, 0, 1]);
        assert_eq!(to_binary(0), [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    pub fn test_four_bit_to_u8() {
        assert_eq!(four_bit_to_u8([1, 1, 1, 1]), 15);
        assert_eq!(four_bit_to_u8([0, 0, 1, 0]), 2);
        assert_eq!(four_bit_to_u8([0, 0, 0, 0]), 0);
    }

    #[test]
    pub fn test_parser_question() {
        let hex_query = "349e010000010000000000000377777706676f6f676c650264650000010001";
        let hex_query = hex::decode(hex_query).unwrap();
        let dns = DNS::new(hex_query).unwrap();

        assert_eq!(dns, DNS {
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
                qtype: String::from("A"),
                qclass: String::from("IN")
            }],
            resource_records: Vec::new()
        });
    }

    #[test]
    pub fn test_parser_response() {
        let hex_response = "349e818000010001000000000377777706676f6f676c650264650000010001c00c00010001000000ee0004acd9a8c3";
        let hex_query = hex::decode(hex_response).unwrap();
        let dns = DNS::new(hex_query).unwrap();

        assert_eq!(dns, DNS {
            id: 13470,
            qr: 1,
            opcode: 0,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 0,
            z: 0,
            rcode: 0,
            qdcount: 1,
            ancount: 1,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.google.de"),
                qtype: String::from("A"),
                qclass: String::from("IN")
            }],
            resource_records: vec![
                ResourceRecord {
                    name: String::from("www.google.de"),
                    rtype: String::from("A"),
                    class: String::from("IN"),
                    ttl: 238,
                    rdlength: 4,
                    rdata: vec![172, 217, 168, 195]
                }
            ]
        });
    }
}
