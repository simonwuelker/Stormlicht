use std::{iter, ops};

cfg_match! {
    cfg(target_pointer_width = "64") => {
        pub type Digit = u64;
        pub type BigDigit = u128;
    }
    cfg(target_pointer_width = "32") => {
        pub type Digit = u32;
        pub type BigDigit = u64;
    }
    _ => {
        compile_error!("Arbitrary sized integers are only available for 32/64 bit platforms");
    }
}

const POWERS: [(Digit, usize); 256] = {
    let mut powers = [(0, 0); 256];

    let mut radix = 2;
    while radix < 256 {
        let mut power = 1;
        let mut base: Digit = radix;

        while let Some(new_base) = base.checked_mul(radix) {
            base = new_base;
            power += 1;
        }

        powers[radix as usize] = (base, power);
        radix += 1;
    }

    powers
};

/// A dynamically sized unsigned integer type
#[derive(Clone, Debug)]
pub struct BigNum {
    /// The least significant digit comes first
    digits: Vec<Digit>,
}

impl BigNum {
    #[must_use]
    pub fn new(number: &str) -> Self {
        if let Some(without_prefix) = number.strip_prefix("0b") {
            return Self::new_with_radix(&to_radix(without_prefix, 2), 2);
        }

        if let Some(without_prefix) = number.strip_prefix("0o") {
            return Self::new_with_radix(&to_radix(without_prefix, 8), 8);
        }

        if let Some(without_prefix) = number.strip_prefix("0x") {
            return Self::new_with_radix(&to_radix(without_prefix, 16), 16);
        }

        Self::new_with_radix(&to_radix(number, 10), 10)
    }

    /// Utility function for the [BigNum] value `0`
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        Self::from_digits(vec![0])
    }

    #[inline]
    #[must_use]
    pub fn from_digits(digits: Vec<Digit>) -> Self {
        Self { digits }
    }

    /// Parse from big-endian digits
    pub fn new_with_radix(digits: &[u32], radix: u32) -> Self {
        if digits.is_empty() {
            return Self::zero();
        }

        debug_assert!(digits.iter().all(|&v| v < radix));

        // Split the digits into chunks
        let (base, power) = POWERS[radix as usize];

        let head_len = if digits.len() % power == 0 {
            power
        } else {
            digits.len() % power
        };

        let (head, tail) = digits.split_at(head_len);

        let mut result = Self::from_digits(vec![]);
        let first = head.iter().fold(0, |acc, &digit| {
            acc * Digit::from(radix) + Digit::from(digit)
        });
        result.digits.push(first);

        let exact_chunks = tail.chunks_exact(power);
        debug_assert!(exact_chunks.remainder().is_empty());

        for chunk in exact_chunks {
            result.digits.push(0);

            let mut carry: BigDigit = 0;
            for d in result.digits_mut() {
                carry += BigDigit::from(*d) * BigDigit::from(base);
                *d = carry as Digit;
                carry >>= Digit::BITS;
            }

            assert_eq!(carry, 0);

            result = result + Digit::from(chunk.iter().fold(0, |acc, &digit| acc * radix + digit));
        }

        result
    }

    /// Try to shrink the internal vector as much as possible by
    /// deallocating unused capacity and removing leading zeros.
    #[inline]
    pub fn compact(&mut self) {
        self.digits.truncate(self.first_nonzero_digit() + 1);
    }

    #[inline]
    #[must_use]
    pub fn digits(&self) -> &[Digit] {
        &self.digits
    }

    #[inline]
    #[must_use]
    fn digits_mut(&mut self) -> &mut [Digit] {
        &mut self.digits
    }

    fn first_nonzero_digit(&self) -> usize {
        self.digits()
            .iter()
            .enumerate()
            .rev()
            .find_map(
                |(index, digit)| {
                    if *digit == 0 {
                        None
                    } else {
                        Some(index)
                    }
                },
            )
            .unwrap_or_default()
    }
}

impl ops::Add for BigNum {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        // Attempt to reuse the storage from the larger argument
        let (mut destination, other) = if self.digits.capacity() < other.digits.capacity() {
            (other, self)
        } else {
            (self, other)
        };

        let max_digits = if destination.digits().len() < other.digits().len() {
            // Reserve the maximum space that the result can take up
            // This might not be a reallocation since we chose the
            // vector with a larger capacity earlier.
            destination.digits.resize(other.digits.len() + 1, 0);
            other.digits.len()
        } else {
            destination.digits.len()
        };

        let mut carry = 0;
        for (d1, &d2) in destination
            .digits_mut()
            .iter_mut()
            .zip(other.digits().iter().chain(iter::repeat(&0)))
            .take(max_digits)
        {
            let (immediate_result, did_overflow) = d1.overflowing_add(d2);
            let (immediate_result, did_overflow_2) = immediate_result.overflowing_add(carry);

            if did_overflow || did_overflow_2 {
                carry = 1;
                *d1 = immediate_result - Digit::MAX;
            } else {
                carry = 0;
                *d1 = immediate_result;
            }
        }
        destination.compact(); // if we allocated too much, free the space
        destination
    }
}

impl ops::Add<Digit> for BigNum {
    type Output = Self;

    fn add(self, mut other: Digit) -> Self::Output {
        let mut result = self; // reuse the already-allocated storage
        let mut add_to_index = 0;

        let mut done = false;
        while !done {
            let (intermediate_result, did_overflow) =
                result.digits[add_to_index].overflowing_add(other);
            result.digits[add_to_index] = intermediate_result;

            if did_overflow {
                other = 1;
                add_to_index += 1;
            } else {
                done = true;
            }
        }
        result
    }
}

impl PartialEq for BigNum {
    fn eq(&self, other: &Self) -> bool {
        // Ignore leading zeros in the comparison
        let up_to_a = self.first_nonzero_digit();
        let up_to_b = other.first_nonzero_digit();

        if up_to_a != up_to_b {
            return false;
        }

        self.digits()
            .iter()
            .zip(other.digits())
            .take(up_to_a)
            .all(|(a, b)| a == b)
    }
}

// Takes an ascii string and converts it to a sequence of digits in the given
// radix and removes leading zeros So `"01_23F"` in base 16 becomes `[1, 2, 3, 15]`.
//
// # Panic
// This function panics if any character is not a valid number for the given base
fn to_radix(number_with_leading_zeros: &str, base: u32) -> Vec<u32> {
    let number = number_with_leading_zeros.trim_start_matches('0');
    let mut digits = Vec::with_capacity(number.len());
    for c in number.chars().filter(|&c| c != '_') {
        digits.push(c.to_digit(base).expect("Digit invalid for given base"));
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::BigNum;

    #[test]
    fn test_equal() {
        let a = BigNum::new("123");
        let b = BigNum::new("123");
        let c = BigNum::new("0123");
        let d = BigNum::new("321");

        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_ne!(c, d);
    }

    #[test]
    fn test_different_radix() {
        let base_10 = BigNum::new("234793475345234234");
        let base_16 = BigNum::new("0x342276BFD88393A");
        let base_8 = BigNum::new("0o15021166577542034472");
        let base_2 = BigNum::new("0b1101000010001001110110101111111101100010000011100100111010");

        assert_eq!(base_10, base_16);
        assert_eq!(base_16, base_8);
        assert_eq!(base_8, base_2);
    }

    #[test]
    fn test_add() {
        let a = BigNum::new("45600000000000000000000000000000000000000999");
        let b = BigNum::new("12300000000000000000000000000000000000000456");
        let d = BigNum::new("57900000000000000000000000000000000000001455");

        assert_eq!(a + b, d);
    }
}
