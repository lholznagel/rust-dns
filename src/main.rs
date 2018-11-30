use failure::Error;
use std::io::{Cursor, Read};
use std::mem;

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

#[derive(Debug)]
pub struct Parser<'a> {
    buffer: Cursor<&'a [u8]>,
}

impl<'a> Parser<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            buffer: Cursor::new(bytes),
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        let mut buffer = [0; 1];
        self.buffer.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        let mut buffer = [0; 2];
        self.buffer.read_exact(&mut buffer)?;
        Ok(unsafe {
            mem::transmute::<[u8; 2], u16>(buffer)
        })
    }

    pub fn read_length(&mut self, length: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; length];
        self.buffer.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn debug(self) -> Self {
        println!("{:?}", self.buffer);
        self
    }
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
    pub questions: Vec<Question>
}

#[derive(Debug, Eq, PartialEq)]
struct Question {
    pub qname: String,
    pub qtype: String,
    pub qclass: String
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
    pub fn test_parser() {
        let hex_query = "dc400100000100000000000006676f6f676c650264650000010001";
        let _hex_response = "dc408180000100010000000006676f6f676c650264650000010001c00c00010001000000280004d83ad203";

        let hex_query = hex::decode(hex_query).unwrap();
        let mut parser = Parser::new(&hex_query);
        let id = parser.read_u16().unwrap().to_be();

        let whatever = parser.read_u8().unwrap();
        let whatever_binary = to_binary(whatever);

        let opcode_bin = [whatever_binary[1], whatever_binary[2], whatever_binary[3], whatever_binary[4]];
        let qr = whatever_binary[0];
        let opcode = four_bit_to_u8(opcode_bin);
        let aa = whatever_binary[5];
        let tc = whatever_binary[6];
        let rd = whatever_binary[7];

        let whatever2 = parser.read_u8().unwrap();
        let whatever2_binary = to_binary(whatever2);

        let rcode_bin = [whatever2_binary[4], whatever2_binary[5], whatever2_binary[6], whatever2_binary[7]];
        let ra = whatever2_binary[0];
        let z = 0;
        let rcode = four_bit_to_u8(rcode_bin);

        let qdcount = parser.read_u16().unwrap().to_be();
        let ancount = parser.read_u16().unwrap().to_be();
        let nscount = parser.read_u16().unwrap().to_be();
        let arcount = parser.read_u16().unwrap().to_be();

        let mut questions = Vec::new();

        let mut question_length = parser.read_u8().unwrap();
        let mut qname = Vec::new();

        while question_length != 0 {
            let mut name = parser.read_length(question_length as usize).unwrap();
            qname.append(&mut name);
            question_length = parser.read_u8().unwrap();

            if question_length != 0 {
                qname.push(46);
            }
        }

        let qname = String::from_utf8(qname).unwrap();
        let qtype = String::from("A"); // TODO write parser
        let qclass = String::from("IN"); // TODO write parser

        questions.push(Question {
            qname,
            qtype,
            qclass
        });

        let dns = DNS {
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
            questions
        };
        assert_eq!(dns, DNS {
            id: 56384,
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
                qname: String::from("google.de"),
                qtype: String::from("A"),
                qclass: String::from("IN")
            }]
        });
    }
}