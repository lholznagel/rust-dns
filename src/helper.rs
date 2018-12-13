pub fn to_binary(val: u8) -> [u8; 8] {
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

#[cfg(test)]
mod tests {
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
}