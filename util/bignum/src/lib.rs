use std::ops::Add;

#[cfg(target_pointer_width = "32")]
type Digit = u32;
#[cfg(target_pointer_width = "32")]
type BigDigit = u64;

#[cfg(target_pointer_width = "64")]
type Digit = u64;
#[cfg(target_pointer_width = "64")]
type BigDigit = u128;

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
/// Digits are stored in little endian
#[derive(Debug)]
pub struct BigNum(Vec<Digit>);

impl BigNum {
    pub fn new(number: &str) -> Self {
        if number.starts_with("0o") {
            Self::new_with_radix(&to_radix(&number[2..], 8), 8)
        } else if number.starts_with("0x") {
            Self::new_with_radix(&to_radix(&number[2..], 16), 16)
        } else {
            Self::new_with_radix(&to_radix(number, 10), 10)
        }
    }

    /// Utility function for the [BigNum] value `0`
    pub fn zero() -> Self {
        Self(vec![0])
    }

    /// Parse from big-endian digits
    pub fn new_with_radix(digits: &[u32], radix: u32) -> Self {
        if digits.is_empty() {
            return Self::zero();
        }

        debug_assert!(digits.iter().all(|&v| v < radix));

        // Split the digits into chunks
        let (base, power) = POWERS[radix as usize];
        dbg!(base, power);
        let head_len = if digits.len() % power == 0 {
            power
        } else {
            digits.len() % power
        };

        let (head, tail) = digits.split_at(head_len);

        let mut result = Self(Vec::new());
        let first = Digit::from(head.iter().fold(0, |acc, &digit| acc * radix + digit));
        result.0.push(first);

        let exact_chunks = tail.chunks_exact(power);
        debug_assert!(exact_chunks.remainder().len() == 0);

        for chunk in exact_chunks {
            dbg!(&result.0);
            result.0.push(0);

            let mut carry: BigDigit = 0;
            for d in result.0.iter_mut() {
                carry += BigDigit::from(*d) * BigDigit::from(base);
                *d = carry as Digit;
                carry >>= Digit::BITS;
            }

            assert_eq!(carry, 0);

            result = result + Digit::from(chunk.iter().fold(0, |acc, &digit| acc * radix + digit));
        }

        result
    }

    fn digits(&self) -> usize {
        self.0.len()
    }

    fn nth_digit(&self, index: usize) -> Option<Digit> {
        if index < self.digits() {
            Some(self.0[index])
        } else {
            None
        }
    }

    fn set_nth_digit(&mut self, index: usize, digit: Digit) {
        if index < self.digits() {
            self.0[index] = digit;
        } else {
            self.0.resize(index + 1, 0);
            self.0[index] = digit;
        }
    }

    fn first_nonzero_digit(&self) -> usize {
        self.0
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

impl Add for BigNum {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        // Attempt to reuse the storage from the larger argument
        let (mut destination, other) = if self.0.capacity() < other.0.capacity() {
            (other, self)
        } else {
            (self, other)
        };

        let max_digits = std::cmp::max(destination.digits(), other.digits());

        let mut carry = 0;
        for i in 0..max_digits {
            let immediate_result = destination.nth_digit(i).unwrap_or_default()
                + other.nth_digit(i).unwrap_or_default()
                + carry;

            if Digit::MAX < immediate_result {
                carry = 1;
                destination.set_nth_digit(i, immediate_result - Digit::MAX);
            } else {
                carry = 0;
                destination.set_nth_digit(i, immediate_result);
            }
        }

        destination
    }
}

impl Add<Digit> for BigNum {
    type Output = Self;

    fn add(self, mut other: Digit) -> Self::Output {
        let mut result = self; // reuse the already-allocated storage
        let mut add_to_index = 0;

        let mut done = false;
        while !done {
            let (intermediate_result, did_overflow) = result.0[add_to_index].overflowing_add(other);
            result.0[add_to_index] = intermediate_result;

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

        for i in 0..up_to_a + 1 {
            if self.nth_digit(i) != other.nth_digit(i) {
                return false;
            }
        }
        true
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
    fn test_add() {
        let a = BigNum::new("45600000000000000000000000000000000000000123");
        let b = BigNum::new("12300000000000000000000000000000000000000456");
        let d = BigNum::new("57900000000000000000000000000000000000000579");

        assert_eq!(a + b, d);
    }
}
