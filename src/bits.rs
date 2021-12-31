
/// A helper for reading bits from a stream of bytes.
///
/// The bit order is most-significant bit first, both inside of individual bytes, and between
/// bytes (ie this simulates an arbitrarily long binary string where the first bit is the
/// highest).
pub struct Bitstream<'a> {
    bytes: &'a [u8],
    bits_left_in_next_byte: u8,
}

impl<'a> Bitstream<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bits_left_in_next_byte: 8,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn num_remaining_bits(&self) -> usize {
        if self.bytes.is_empty() {
            0
        } else {
            self.bits_left_in_next_byte as usize + (self.bytes.len() - 1) * 8
        }
    }

    /// Peaks at the next n bits, where n <= 16.
    /// Returns None if there are not enough bits left.
    pub fn peek_n_bits(&self, n: u8) -> Option<u16> {
        assert!(n <= 16);

        let mut n = n;
        let mut res = 0u16;

        // We'll have to look at (at most) 3 bytes (eg when asked for 16 bits, and there's only 1
        // left in the current byte).
        for i in 0..3 {
            let bits_left = if i == 0 { self.bits_left_in_next_byte } else { 8 };
            // Read the byte in question, and shift it towards the most-significant bits if it's
            // partial.
            let this_byte = self.bytes.get(i)? << (8-bits_left);

            if bits_left >= n {
                // Read n bits, starting from the most significant bits.
                res <<= n;
                let this_chunk = this_byte >> (8-n);
                res |= this_chunk as u16;
                return Some(res);
            }

            // Read bits_left bits, starting from the most significant bits.
            res <<= bits_left;
            let this_chunk = this_byte >> (8-bits_left);
            res |= this_chunk as u16;
            n -= bits_left;
        }

        panic!("BUG: should never be reached!");
    }

    /// Pops off the next n bits.
    pub fn pop_n_bits(&mut self, n: u8) {
        let mut n = n;
        while n > 0 {
            if self.bits_left_in_next_byte <= n {
                // Pop the whole byte.
                n -= self.bits_left_in_next_byte;
                self.bytes = &self.bytes[1..];
                self.bits_left_in_next_byte = 8;
            } else {
                // Discard part of the byte.
                self.bits_left_in_next_byte -= n;
                break;
            }
        }
    }

    /// Reads the next n bits, where n <= 16, and pops them off.
    pub fn get_n_bits(&mut self, n: u8) -> Option<u16> {
        let res = self.peek_n_bits(n)?;
        self.pop_n_bits(n);
        Some(res)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bitstream() {
        let mut bits = super::Bitstream::new(&[0b1101_0010, 0b1111_1110, 0b0010_1000]);

        assert_eq!(bits.num_remaining_bits(), 24);
        assert!(!bits.is_empty());

        assert_eq!(bits.peek_n_bits(1), Some(0b1));
        assert_eq!(bits.peek_n_bits(2), Some(0b11));
        assert_eq!(bits.peek_n_bits(3), Some(0b110));
        assert_eq!(bits.peek_n_bits(4), Some(0b1101));
        assert_eq!(bits.peek_n_bits(5), Some(0b1101_0));
        assert_eq!(bits.peek_n_bits(6), Some(0b1101_00));
        assert_eq!(bits.peek_n_bits(7), Some(0b1101_001));
        assert_eq!(bits.peek_n_bits(8), Some(0b1101_0010));
        assert_eq!(bits.peek_n_bits(9), Some(0b1101_0010_1));
        assert_eq!(bits.peek_n_bits(10), Some(0b1101_0010_11));
        assert_eq!(bits.peek_n_bits(11), Some(0b1101_0010_111));
        assert_eq!(bits.peek_n_bits(12), Some(0b1101_0010_1111));
        assert_eq!(bits.peek_n_bits(13), Some(0b1101_0010_1111_1));
        assert_eq!(bits.peek_n_bits(14), Some(0b1101_0010_1111_11));
        assert_eq!(bits.peek_n_bits(15), Some(0b1101_0010_1111_111));
        assert_eq!(bits.peek_n_bits(16), Some(0b1101_0010_1111_1110));

        assert_eq!(bits.get_n_bits(6), Some(0b1101_00));
        assert_eq!(bits.get_n_bits(5), Some(0b10_111));
        assert_eq!(bits.num_remaining_bits(), 13);
        assert_eq!(bits.get_n_bits(16), None);
        assert_eq!(bits.get_n_bits(13), Some(0b1_1110_0010_1000));

        assert!(bits.is_empty());
        assert_eq!(bits.num_remaining_bits(), 0);
    }
}
