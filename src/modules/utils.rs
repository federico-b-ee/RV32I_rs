pub fn u32_to_bitvec(value: u32) -> Vec<u32> {
    let mut bits = Vec::new();
    for i in 0..32 {
        let bit = (value >> i) & 1;
        bits.push(bit);
    }
    bits
}

pub fn bitvec_to_u32(bits: &[u32]) -> u32 {
    let mut value = 0;
    for (i, &bit) in bits.iter().enumerate() {
        value |= bit << i;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_to_bitvec() {
        let mut one = vec![0; 32];
        one[0] = 1;

        let max_u8: Vec<u32> = [vec![1; 8], vec![0; 24]].concat();

        assert_eq!(u32_to_bitvec(0), vec![0; 32]);
        assert_eq!(u32_to_bitvec(1), one);
        assert_eq!(u32_to_bitvec(255), max_u8);
        assert_eq!(u32_to_bitvec(4294967295), vec![1; 32]);
    }

    #[test]
    fn test_bitvec_to_u32() {
        assert_eq!(bitvec_to_u32(&[0; 32]), 0);
        assert_eq!(bitvec_to_u32(&[1]), 1);
        assert_eq!(bitvec_to_u32(&[1; 8]), 255);
        assert_eq!(bitvec_to_u32(&[1; 32]), 4294967295);
    }
}
