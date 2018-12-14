use failure::Error;
use std::io::{Cursor, Read};
use std::mem;

#[derive(Debug)]
pub struct Reader<'a> {
    buffer: Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            buffer: Cursor::new(bytes),
        }
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

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        let mut buffer = [0; 1];
        self.buffer.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        let mut buffer = [0; 2];
        self.buffer.read_exact(&mut buffer)?;

        Ok(unsafe { mem::transmute::<[u8; 2], u16>(buffer) })
    }

    pub fn read_u16_be(&mut self) -> Result<u16, Error> {
        Ok(self.read_u16()?.to_be())
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        let mut buffer = [0; 4];
        self.buffer.read_exact(&mut buffer)?;

        Ok(unsafe { mem::transmute::<[u8; 4], u32>(buffer) })
    }

    pub fn read_u32_be(&mut self) -> Result<u32, Error> {
        Ok(self.read_u32()?.to_be())
    }

    pub fn read_u8_as_binary(&mut self) -> Result<[u8; 8], Error> {
        let mut val = self.read_u8()?;
        let mut result = [0; 8];

        if val >= 128 {
            result[0] = 1;
            val -= 128;
        }

        if val >= 64 {
            result[1] = 1;
            val -= 64;
        }

        if val >= 32 {
            result[2] = 1;
            val -= 32;
        }

        if val >= 16 {
            result[3] = 1;
            val -= 16;
        }

        if val >= 8 {
            result[4] = 1;
            val -= 8;
        }

        if val >= 4 {
            result[5] = 1;
            val -= 4;
        }

        if val >= 2 {
            result[6] = 1;
            val -= 2;
        }

        if val >= 1 {
            result[7] = 1;
        }

        Ok(result)
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    pub fn test_to_binary() {
        assert_eq!(read_u8_as_binary(255), [1, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(read_u8_as_binary(128), [1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(read_u8_as_binary(65), [0, 1, 0, 0, 0, 0, 0, 1]);
        assert_eq!(read_u8_as_binary(0), [0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
