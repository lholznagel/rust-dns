#[derive(Clone, Debug)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
        }
    }

    pub fn write_u8(mut self, value: u8) -> Self {
        self.bytes.push(value);
        self
    }

    pub fn write_binary_as_u8(self, value: [u8; 8]) -> Self {
        let mut val = 0;

        if value[0] == 1 {
            val += 128;
        }

        if value[1] == 1 {
            val += 64;
        }

        if value[2] == 1 {
            val += 32;
        }

        if value[3] == 1 {
            val += 16;
        }

        if value[4] == 1 {
            val += 8;
        }

        if value[5] == 1 {
            val += 4;
        }

        if value[6] == 1 {
            val += 2;
        }

        if value[7] == 1 {
            val += 1;
        }

        self.write_u8(val)
    }

    pub fn write_u16(mut self, value: u16) -> Self {
        let bytes: [u8; 2] = unsafe { ::std::mem::transmute(value) };
        self.bytes.append(&mut bytes.to_vec());
        self
    }

    pub fn write_u16_be(self, value: u16) -> Self {
        self.write_u16(value.to_be())
    }

    pub fn write_u32(mut self, value: u32) -> Self {
        let bytes: [u8; 4] = unsafe { ::std::mem::transmute(value) };
        self.bytes.append(&mut bytes.to_vec());
        self
    }

    pub fn write_u32_be(self, value: u32) -> Self {
        self.write_u32(value.to_be())
    }

    pub fn write_vec(mut self, mut vec: Vec<u8>) -> Self {
        self.bytes.append(&mut vec);
        self
    }

    pub fn position(&self) -> usize {
        self.bytes.len()
    }

    pub fn build(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_u16() {
        let builder = Writer::new().write_u16(57_868).build();
        assert_eq!(builder, [12, 226]);

        let builder = Writer::new().write_u16(34_678).build();
        assert_eq!(builder, [118, 135]);

        let builder = Writer::new().write_u16_be(57_868).build();
        assert_eq!(builder, [226, 12]);

        let builder = Writer::new().write_u16_be(34_678).build();
        assert_eq!(builder, [135, 118]);
    }

    #[test]
    pub fn test_u32() {
        let builder = Writer::new().write_u32(1_257_868).build();
        assert_eq!(builder, [140, 49, 19, 0]);

        let builder = Writer::new().write_u32(167_437_900).build();
        assert_eq!(builder, [76, 230, 250, 9]);

        let builder = Writer::new().write_u32_be(1_257_868).build();
        assert_eq!(builder, [0, 19, 49, 140]);

        let builder = Writer::new().write_u32_be(167_437_900).build();
        assert_eq!(builder, [9, 250, 230, 76]);
    }
}
