use failure::Error;
use std::io::{Cursor, Read};
use std::mem;

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

    fn read_u16(&mut self) -> Result<u16, Error> {
        let mut buffer = [0; 2];
        self.buffer.read_exact(&mut buffer)?;

        Ok(unsafe { mem::transmute::<[u8; 2], u16>(buffer) })
    }

    pub fn read_u16_be(&mut self) -> Result<u16, Error> {
        Ok(self.read_u16()?.to_be())
    }

    pub fn read_u32_be(&mut self) -> Result<u32, Error> {
        let mut buffer = [0; 4];
        self.buffer.read_exact(&mut buffer)?;

        Ok(unsafe { mem::transmute::<[u8; 4], u32>(buffer) }.to_be())
    }

    pub fn read_length(&mut self, length: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; length];
        self.buffer.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn position(&self) -> u64 {
        self.buffer.position()
    }

    pub fn set_position(&mut self, position: u64) {
        self.buffer.set_position(position);
    }
}
