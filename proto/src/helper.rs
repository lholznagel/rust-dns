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

pub fn u8_to_four_bit(val: u8) -> [u8; 4] {
    let mut val = val;
    let mut result = [0; 4];

    if val >= 8 {
        result[0] = 1;
        val -= 8;
    }

    if val >= 4 {
        result[1] = 1;
        val -= 4;
    }

    if val >= 2 {
        result[2] = 1;
        val -= 2;
    }

    if val >= 1 {
        result[3] = 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_four_bit_to_u8() {
        assert_eq!(four_bit_to_u8([1, 1, 1, 1]), 15);
        assert_eq!(four_bit_to_u8([1, 0, 1, 0]), 10);
        assert_eq!(four_bit_to_u8([0, 0, 1, 0]), 2);
        assert_eq!(four_bit_to_u8([0, 0, 0, 0]), 0);
    }

    #[test]
    pub fn test_u8_to_four_bit() {
        assert_eq!(u8_to_four_bit(15), [1, 1, 1, 1]);
        assert_eq!(u8_to_four_bit(10), [1, 0, 1, 0]);
        assert_eq!(u8_to_four_bit(2), [0, 0, 1, 0]);
        assert_eq!(u8_to_four_bit(0), [0, 0, 0, 0]);
    }
}
