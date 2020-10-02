use crate::error::*;

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

    pub fn read_length(&mut self, length: usize) -> Result<Vec<u8>> {
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

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buffer = [0; 1];
        self.buffer.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buffer = [0; 2];
        self.buffer.read_exact(&mut buffer)?;

        Ok(unsafe { mem::transmute::<[u8; 2], u16>(buffer) })
    }

    pub fn read_u16_be(&mut self) -> Result<u16> {
        Ok(self.read_u16()?.to_be())
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buffer = [0; 4];
        self.buffer.read_exact(&mut buffer)?;

        Ok(unsafe { mem::transmute::<[u8; 4], u32>(buffer) })
    }

    pub fn read_u32_be(&mut self) -> Result<u32> {
        Ok(self.read_u32()?.to_be())
    }

    pub fn read_u8_as_binary(&mut self) -> Result<[u8; 8]> {
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

pub trait ByteReader: std::io::Read {
    /// Reads one byte from the byte array and returns it as u8 value
    ///
    /// If there are any errors, a `ProtocolError` is returned
    /// containing the underlying `std::io::Error`
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        let mut buffer = [0; 1];
        self.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    /// Reads two byte from the byte array and returns it as u16 big endian value
    ///
    /// If there are any errors, a `ProtocolError` is returned
    /// containing the underlying `std::io::Error`
    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        let mut buffer = [0; 2];
        self.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    /// Reads four byte from the byte array and returns it as u32 big endian value
    ///
    /// If there are any errors, a `ProtocolError` is returned
    /// containing the underlying `std::io::Error`
    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        let mut buffer = [0; 4];
        self.read_exact(&mut buffer)?;
        Ok(u32::from_be_bytes(buffer))
    }

    /// Reads the given amount of bytes and returns them as a `Vec<u8>`
    ///
    /// If there are any errors, a `ProtocolError` is returned
    /// containing the underlying `std::io::Error`
    #[inline]
    fn read_length(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(length);

        for _ in 0..length {
            buf.push(0);
        }

        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    fn read_binary(&mut self) -> Result<[u8; 8]> {
        let mut val = self.read_u8()?;
        let mut buf = [0u8; 8];

        for i in (0..8).rev() {
            buf[i] = val % 2;
            val = val / 2;

            if val == 0 {
                break;
            }
        }

        Ok(buf)
    }
}

impl<R: std::io::Read + ?Sized> ByteReader for R {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn read_u8_001() {
        let mut reader = Cursor::new(vec![100u8]);
        assert_eq!(reader.read_u8().unwrap(), 100u8);
        assert!(reader.read_u8().is_err());
    }

    #[test]
    fn read_u8_002() {
        let mut reader = Cursor::new(vec![100u8, 100u8]);
        assert_eq!(reader.read_u8().unwrap(), 100u8);
        assert_eq!(reader.read_u8().unwrap(), 100u8);
        assert!(reader.read_u8().is_err());
    }

    #[test]
    fn read_u8_003() {
        let mut reader = Cursor::new(Vec::new());
        assert!(reader.read_u8().is_err());
    }

    #[test]
    fn read_u16_001() {
        let mut reader = Cursor::new(vec![100u8, 100u8]);
        assert_eq!(reader.read_u16().unwrap(), 25700u16);
        assert!(reader.read_u16().is_err());
    }

    #[test]
    fn read_u16_002() {
        let mut reader = Cursor::new(vec![100u8, 100u8, 100u8, 100u8]);
        assert_eq!(reader.read_u16().unwrap(), 25700u16);
        assert_eq!(reader.read_u16().unwrap(), 25700u16);
        assert!(reader.read_u16().is_err());
    }

    #[test]
    fn read_u16_003() {
        let mut reader = Cursor::new(Vec::new());
        assert!(reader.read_u16().is_err());
    }

    #[test]
    fn read_length_001() {
        let mut reader = Cursor::new(vec![100u8, 100u8]);
        assert_eq!(reader.read_length(2usize).unwrap(), vec![100u8, 100u8]);
        assert!(reader.read_length(2usize).is_err());
    }

    #[test]
    fn read_length_002() {
        let mut reader = Cursor::new(vec![100u8, 100u8, 100u8, 100u8]);
        assert_eq!(reader.read_length(2usize).unwrap(), vec![100u8, 100u8]);
        assert_eq!(reader.read_length(2usize).unwrap(), vec![100u8, 100u8]);
        assert!(reader.read_length(2usize).is_err());
    }

    #[test]
    fn read_length_003() {
        let mut reader = Cursor::new(Vec::new());
        assert!(reader.read_length(2usize).is_err());
    }

    #[test]
    pub fn test_to_binary() {
        let mut reader = Cursor::new(vec![255u8]);
        assert_eq!(reader.read_binary().unwrap(), [1, 1, 1, 1, 1, 1, 1, 1]);

        let mut reader = Cursor::new(vec![128u8]);
        assert_eq!(reader.read_binary().unwrap(), [1, 0, 0, 0, 0, 0, 0, 0]);

        let mut reader = Cursor::new(vec![65u8]);
        assert_eq!(reader.read_binary().unwrap(), [0, 1, 0, 0, 0, 0, 0, 1]);

        let mut reader = Cursor::new(vec![0u8]);
        assert_eq!(reader.read_binary().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
