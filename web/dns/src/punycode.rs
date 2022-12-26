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
    if c < 26 {
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
        'A'..='Z' => c as u32 - 'A' as u32,
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

    k + (((BASE - TMIN + 1) * delta) / (delta + SKEW))
}

pub fn punycode_encode(input: &str) -> Result<String, ()> {
    let mut n = INITIAL_N;
    let mut delta: u32 = 0;
    let mut bias = INITIAL_BIAS;
    let num_basic = input.chars().filter(|c| c.is_ascii()).count() as u32;
    let mut h = num_basic;

    let mut output: String = input.chars().filter(|c| c.is_ascii()).collect();
    let input_len = input.chars().count() as u32;
    if num_basic > 0 && num_basic != input_len {
        output.push('-');
    }
    while h < input_len {
        let m = input.chars().filter(|c| *c as u32 >= n).min().unwrap() as u32;
        delta = delta
            .checked_add((m - n).checked_mul(h + 1).ok_or(())?)
            .ok_or(())?;
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

pub fn punycode_decode(input: &str) -> Result<String, ()> {
    if !input.is_ascii() {
        return Err(());
    }

    let (mut output, extended) = match input.rfind('-') {
        Some(i) => {
            if i != input.len() - 1 {
                (input[..i].chars().collect(), &input[i + 1..])
            } else {
                // If there are no trailing special characters, the dash was not a seperator,
                // it was part of the literal ascii str
                (input[..i + 1].chars().collect(), &input[i + 1..])
            }
        },
        None => (vec![], input),
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
            i = i
                .checked_add(digit.checked_mul(weight).ok_or(())?)
                .ok_or(())?;

            let threshold = if k <= bias + TMIN {
                TMIN
            } else if k >= bias + TMAX {
                TMAX
            } else {
                k - bias
            };

            if digit < threshold {
                break;
            }

            weight = weight.checked_mul(BASE - threshold).ok_or(())?;
            k += BASE;
        }

        let num_points = output.len() as u32 + 1;
        bias = adapt(i - old_i, num_points, old_i == 0);
        n = n.checked_add(i / num_points).ok_or(())?;
        i %= num_points;

        output.insert(i as usize, char::try_from(n).map_err(|_| ())?);
        i += 1;
    }
    Ok(output.iter().collect())
}

/// The returned value is guaranteed to be pure ascii
pub fn idna_encode(input: &str) -> Result<String, ()> {
    // Don't encode strings that are already pure ascii
    if input.is_ascii() {
        Ok(input.to_string())
    } else {
        Ok(format!("xn--{}", punycode_encode(input)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://www.rfc-editor.org/rfc/rfc3492#section-7.1
    const ARABIC: &'static str = "\u{0644}\u{064A}\u{0647}\u{0645}\u{0627}\u{0628}\u{062A}\u{0643}\u{0644}\u{0645}\u{0648}\u{0634}\u{0639}\u{0631}\u{0628}\u{064A}\u{061F}";
    const ARABIC_ENCODED: &'static str = "egbpdaj6bu4bxfgehfvwxn";

    const CHINESE: &'static str =
        "\u{4ED6}\u{4EEC}\u{4E3A}\u{4EC0}\u{4E48}\u{4E0D}\u{8BF4}\u{4E2D}\u{6587}";
    const CHINESE_ENCODED: &'static str = "ihqwcrb4cv8a8dqg056pqjye";

    const CHINESE_2: &'static str =
        "\u{4ED6}\u{5011}\u{7232}\u{4EC0}\u{9EBD}\u{4E0D}\u{8AAA}\u{4E2D}\u{6587}";
    const CHINESE_ENCODED_2: &'static str = "ihqwctvzc91f659drss3x8bo0yb";

    const CZECH: &'static str = "\u{0050}\u{0072}\u{006F}\u{010D}\u{0070}\u{0072}\u{006F}\u{0073}\u{0074}\u{011B}\u{006E}\u{0065}\u{006D}\u{006C}\u{0075}\u{0076}\u{00ED}\u{010D}\u{0065}\u{0073}\u{006B}\u{0079}";
    const CZECH_ENCODED: &'static str = "Proprostnemluvesky-uyb24dma41a";

    const HEBREW: &'static str = "\u{05DC}\u{05DE}\u{05D4}\u{05D4}\u{05DD}\u{05E4}\u{05E9}\u{05D5}\u{05D8}\u{05DC}\u{05D0}\u{05DE}\u{05D3}\u{05D1}\u{05E8}\u{05D9}\u{05DD}\u{05E2}\u{05D1}\u{05E8}\u{05D9}\u{05EA}";
    const HEBREW_ENCODED: &'static str = "4dbcagdahymbxekheh6e0a7fei0b";

    const HINDI: &'static str = "\u{092F}\u{0939}\u{0932}\u{094B}\u{0917}\u{0939}\u{093F}\u{0928}\u{094D}\u{0926}\u{0940}\u{0915}\u{094D}\u{092F}\u{094B}\u{0902}\u{0928}\u{0939}\u{0940}\u{0902}\u{092C}\u{094B}\u{0932}\u{0938}\u{0915}\u{0924}\u{0947}\u{0939}\u{0948}\u{0902}";
    const HINDI_ENCODED: &'static str = "i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd";

    const JAPANESE: &'static str = "\u{306A}\u{305C}\u{307F}\u{3093}\u{306A}\u{65E5}\u{672C}\u{8A9E}\u{3092}\u{8A71}\u{3057}\u{3066}\u{304F}\u{308C}\u{306A}\u{3044}\u{306E}\u{304B}";
    const JAPANESE_ENCODED: &'static str = "n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa";

    const KOREAN: &'static str = "\u{C138}\u{ACC4}\u{C758}\u{BAA8}\u{B4E0}\u{C0AC}\u{B78C}\u{B4E4}\u{C774}\u{D55C}\u{AD6D}\u{C5B4}\u{B97C}\u{C774}\u{D574}\u{D55C}\u{B2E4}\u{BA74}\u{C5BC}\u{B9C8}\u{B098}\u{C88B}\u{C744}\u{AE4C}";
    const KOREAN_ENCODED: &'static str =
        "989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c";

    // NOTE: this spec version of test includes an uppercase "D"
    // We don't support capitalized letters in the non-basic text (at least not during encoding)
    // Therefore, i replaced it with a lowercase "d"
    const RUSSIAN: &'static str = "\u{043F}\u{043E}\u{0447}\u{0435}\u{043C}\u{0443}\u{0436}\u{0435}\u{043E}\u{043D}\u{0438}\u{043D}\u{0435}\u{0433}\u{043E}\u{0432}\u{043E}\u{0440}\u{044F}\u{0442}\u{043F}\u{043E}\u{0440}\u{0443}\u{0441}\u{0441}\u{043A}\u{0438}";
    const RUSSIAN_ENCODED: &'static str = "b1abfaaepdrnnbgefbadotcwatmq2g4l";

    const SPANISH: &'static str = "Porqu\u{00E9}nopuedensimplementehablarenEspa\u{00F1}ol";
    const SPANISH_ENCODED: &'static str = "PorqunopuedensimplementehablarenEspaol-fmd56a";

    const VIETNAMESE: &'static str =
        "T\u{1EA1}isaoh\u{1ECD}kh\u{00F4}ngth\u{1EC3}ch\u{1EC9}n\u{00F3}iti\u{1EBF}ngVi\u{1EC7}t";
    const VIETNAMESE_ENCODED: &'static str = "TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g";

    const JAPANESE_2: &'static str = "3\u{5E74}B\u{7D44}\u{91D1}\u{516B}\u{5148}\u{751F}";
    const JAPANESE_ENCODED_2: &'static str = "3B-ww4c5e180e575a65lsy2b";

    const JAPANESE_3: &'static str = "\u{5B89}\u{5BA4}\u{5948}\u{7F8E}\u{6075}-with-SUPER-MONKEYS";
    const JAPANESE_ENCODED_3: &'static str = "-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n";

    const JAPANESE_4: &'static str =
        "Hello-Another-Way-\u{305D}\u{308C}\u{305E}\u{308C}\u{306E}\u{5834}\u{6240}";
    const JAPANESE_ENCODED_4: &'static str = "Hello-Another-Way--fc4qua05auwb3674vfr0b";

    const JAPANESE_5: &'static str = "\u{3072}\u{3068}\u{3064}\u{5C4B}\u{6839}\u{306E}\u{4E0B}2";
    const JAPANESE_ENCODED_5: &'static str = "2-u9tlzr9756bt3uc0v";

    const JAPANESE_6: &'static str = "Maji\u{3067}Koi\u{3059}\u{308B}5\u{79D2}\u{524D}";
    const JAPANESE_ENCODED_6: &'static str = "MajiKoi5-783gue6qz075azm5e";

    const JAPANESE_7: &'static str = "\u{30D1}\u{30D5}\u{30A3}\u{30FC}de\u{30EB}\u{30F3}\u{30D0}";
    const JAPANESE_ENCODED_7: &'static str = "de-jg4avhby1noc0d";

    const JAPANESE_8: &'static str = "\u{305D}\u{306E}\u{30B9}\u{30D4}\u{30FC}\u{30C9}\u{3067}";
    const JAPANESE_ENCODED_8: &'static str = "d9juau41awczczp";

    const PURE_ASCII: &'static str = "-> $1.00 <-";
    const PURE_ASCII_ENCODED: &'static str = "-> $1.00 <-";

    #[test]
    fn test_punycode_decode() {
        assert_eq!(punycode_decode(ARABIC_ENCODED).as_deref(), Ok(ARABIC));
        assert_eq!(punycode_decode(CHINESE_ENCODED).as_deref(), Ok(CHINESE));
        assert_eq!(punycode_decode(CHINESE_ENCODED_2).as_deref(), Ok(CHINESE_2));
        assert_eq!(punycode_decode(CZECH_ENCODED).as_deref(), Ok(CZECH));
        assert_eq!(punycode_decode(HEBREW_ENCODED).as_deref(), Ok(HEBREW));
        assert_eq!(punycode_decode(HINDI_ENCODED).as_deref(), Ok(HINDI));
        assert_eq!(punycode_decode(JAPANESE_ENCODED).as_deref(), Ok(JAPANESE));
        assert_eq!(punycode_decode(KOREAN_ENCODED).as_deref(), Ok(KOREAN));
        assert_eq!(punycode_decode(RUSSIAN_ENCODED).as_deref(), Ok(RUSSIAN));
        assert_eq!(punycode_decode(SPANISH_ENCODED).as_deref(), Ok(SPANISH));
        assert_eq!(
            punycode_decode(VIETNAMESE_ENCODED).as_deref(),
            Ok(VIETNAMESE)
        );
        assert_eq!(
            punycode_decode(JAPANESE_ENCODED_2).as_deref(),
            Ok(JAPANESE_2)
        );
        assert_eq!(
            punycode_decode(JAPANESE_ENCODED_3).as_deref(),
            Ok(JAPANESE_3)
        );
        assert_eq!(
            punycode_decode(JAPANESE_ENCODED_4).as_deref(),
            Ok(JAPANESE_4)
        );
        assert_eq!(
            punycode_decode(JAPANESE_ENCODED_5).as_deref(),
            Ok(JAPANESE_5)
        );
        assert_eq!(
            punycode_decode(JAPANESE_ENCODED_6).as_deref(),
            Ok(JAPANESE_6)
        );
        assert_eq!(
            punycode_decode(JAPANESE_ENCODED_7).as_deref(),
            Ok(JAPANESE_7)
        );
        assert_eq!(
            punycode_decode(JAPANESE_ENCODED_8).as_deref(),
            Ok(JAPANESE_8)
        );
        assert_eq!(
            punycode_decode(PURE_ASCII_ENCODED).as_deref(),
            Ok(PURE_ASCII)
        );
    }

    #[test]
    fn test_punycode_encode() {
        assert_eq!(punycode_encode(ARABIC).as_deref(), Ok(ARABIC_ENCODED));
        assert_eq!(punycode_encode(CHINESE).as_deref(), Ok(CHINESE_ENCODED));
        assert_eq!(punycode_encode(CHINESE_2).as_deref(), Ok(CHINESE_ENCODED_2));
        assert_eq!(punycode_encode(CZECH).as_deref(), Ok(CZECH_ENCODED));
        assert_eq!(punycode_encode(HEBREW).as_deref(), Ok(HEBREW_ENCODED));
        assert_eq!(punycode_encode(HINDI).as_deref(), Ok(HINDI_ENCODED));
        assert_eq!(punycode_encode(JAPANESE).as_deref(), Ok(JAPANESE_ENCODED));
        assert_eq!(punycode_encode(KOREAN).as_deref(), Ok(KOREAN_ENCODED));
        assert_eq!(punycode_encode(RUSSIAN).as_deref(), Ok(RUSSIAN_ENCODED));
        assert_eq!(punycode_encode(SPANISH).as_deref(), Ok(SPANISH_ENCODED));
        assert_eq!(
            punycode_encode(VIETNAMESE).as_deref(),
            Ok(VIETNAMESE_ENCODED)
        );
        assert_eq!(
            punycode_encode(JAPANESE_2).as_deref(),
            Ok(JAPANESE_ENCODED_2)
        );
        assert_eq!(
            punycode_encode(JAPANESE_3).as_deref(),
            Ok(JAPANESE_ENCODED_3)
        );
        assert_eq!(
            punycode_encode(JAPANESE_4).as_deref(),
            Ok(JAPANESE_ENCODED_4)
        );
        assert_eq!(
            punycode_encode(JAPANESE_5).as_deref(),
            Ok(JAPANESE_ENCODED_5)
        );
        assert_eq!(
            punycode_encode(JAPANESE_6).as_deref(),
            Ok(JAPANESE_ENCODED_6)
        );
        assert_eq!(
            punycode_encode(JAPANESE_7).as_deref(),
            Ok(JAPANESE_ENCODED_7)
        );
        assert_eq!(
            punycode_encode(JAPANESE_8).as_deref(),
            Ok(JAPANESE_ENCODED_8)
        );
        assert_eq!(
            punycode_encode(PURE_ASCII).as_deref(),
            Ok(PURE_ASCII_ENCODED)
        );
    }
}
