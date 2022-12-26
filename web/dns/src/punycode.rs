//! Punycode implementation as per <https://www.rfc-editor.org/rfc/rfc3492>
//!
//! Also [IDNA](https://de.wikipedia.org/wiki/Internationalizing_Domain_Names_in_Applications)

const BASE: u32 = 36;
const TMIN: u32 = 1;
const TMAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;

fn encode_digit(c: u32) -> char {
	debug_assert!(c < BASE);
	if  c < 26 {
		// a..z
		char::try_from(c + 97).unwrap()
	} else {
		// 0..9
		char::try_from(c + 22).unwrap()
	}
}

/// Panics if the character is not a valid lowercase base-36 digit
fn decode_digit(c: char) -> u32 {
	match c {
		'a'..='z' => c as u32 - 'a' as u32,
		'0'..='9' => c as u32 - '0' as u32 + 26,
		_ => panic!("Invalid base 36 digit: {c}"),
	}
}

fn adapt(mut delta: u32, num_points: u32, is_first: bool) -> u32 {
	delta /= if is_first { DAMP } else { 2 };

	delta += delta / num_points;
	let mut k = 0;

	while delta > ((BASE - TMIN) * TMAX) / 2 {
		delta /= BASE - TMIN;
		k += BASE;
	}

	(BASE * k) + (((BASE - TMIN + 1) * delta) / (delta + SKEW))
}

pub fn encode(input: &str) -> Result<String, ()> {
	let mut n = INITIAL_N;
	let mut delta: u32 = 0;
	let mut bias = INITIAL_BIAS;
	let num_basic = input.chars().filter(|c| c.is_ascii()).count() as u32;
	let mut h = num_basic;

	let mut output: String = input.chars().filter(|c| c.is_ascii()).collect();
	if num_basic > 0 {
		output.push('-');
	}
	while h < input.chars().count() as u32 {
		let m = input.chars().filter(|c| *c as u32 >= n).min().unwrap() as u32;
		delta = delta.checked_add((m - n).checked_mul(h + 1).ok_or(())?).ok_or(())?;
		n = m;

		for c in input.chars().map(|c| c as u32) {
			if c < n {
				delta += 1;
			}

			if c == n {
				let mut q = delta;

				let mut k = BASE;
				loop {
					let threshold = if k <= bias + TMIN {
						TMIN
					} else if k >= bias + TMAX {
						TMAX
					} else {
						k - bias
					};

					if q < threshold {
						break;
					}
					let codepoint_numeric = threshold + ((q - threshold) % (BASE - threshold));
					output.push(encode_digit(codepoint_numeric));

					q = (q - threshold) / (BASE - threshold);
					k += BASE;
				}

				output.push(encode_digit(q));
				bias = adapt(delta, h + 1, h == num_basic);
				delta = 0;
				h += 1;
			}
		}
		delta += 1;
		n += 1;
	}
	Ok(output)
}

pub fn decode(input: &str) -> Result<String, ()> {
	if !input.is_ascii() {
		return Err(());
	}

	let (mut output, extended) = match input.rfind('-') {
		Some(i) => {
			(input[..i].to_string(), &input[i + 1..])
		}
		None => {
			(String::new(), input)
		}
	};

	let mut n = INITIAL_N;
	let mut i: u32 = 0;
	let mut bias = INITIAL_BIAS;

	let mut codepoints = extended.chars().peekable();
	while codepoints.peek().is_some() {
		let old_i = i;
		let mut weight = 1;
		let mut k = BASE;
		loop {
			let code_point = codepoints.next().ok_or(())?;
			let digit = decode_digit(code_point);
			i = i.checked_add(digit.checked_mul(weight).ok_or(())?).ok_or(())?;

			let threshold = if k <= bias + TMIN {
				TMIN
			} else if k >= bias + TMAX {
				TMAX
			} else {
				k - bias
			};

			if digit < threshold {
				break
			}

			weight = weight.checked_mul(BASE - threshold).ok_or(())?;
			k += BASE;
		}

		let num_points = output.chars().count() as u32 + 1;
		bias = adapt(i - old_i, num_points, old_i == 0);
		n = n.checked_add(i / num_points).ok_or(())?;
		i %= num_points;

		output.insert(i as usize, char::try_from(n).unwrap());
		i += 1;
	}
	Ok(output)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decode() {
		assert_eq!(decode("acadmie-franaise-npb1a").unwrap(), "académie-française");
	}

	#[test]
	fn test_encode() {
		assert_eq!(encode("académie-française").unwrap(), "acadmie-franaise-npb1a");
	}
}