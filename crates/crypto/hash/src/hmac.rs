use std::cmp::Ordering;

use crate::CryptographicHashAlgorithm;

pub(crate) fn hmac<T: CryptographicHashAlgorithm>(
    key: &[u8],
    data: &[u8],
) -> [u8; T::BLOCK_SIZE_OUT]
// NOTE: This is a meaningless trait bound, necessary to work around quirks
//       in the incomplete generic_const_exprs feature.
//       If you see this, try removing it to see if its still necessary (:
//       (tested on rustc 1.76.0-nightly (3f28fe133 2023-12-18))
where
    [(); T::BLOCK_SIZE_IN]: Sized,
{
    let k0: [u8; T::BLOCK_SIZE_IN] = match key.len().cmp(&T::BLOCK_SIZE_IN) {
        Ordering::Less => {
            // Pad key with zeros on the right
            let mut padded_key = [0; T::BLOCK_SIZE_IN];
            padded_key[..key.len()].copy_from_slice(key);
            padded_key
        },
        Ordering::Equal => key.try_into().expect("key size is equal to block size"),
        Ordering::Greater => {
            // Hash the key and pad the result with zeros on the right to the required size
            let mut padded_key = [0; T::BLOCK_SIZE_IN];
            padded_key[..T::BLOCK_SIZE_OUT].copy_from_slice(&T::hash(key));
            padded_key
        },
    };

    let k0_xor_opad = k0.map(|e| e ^ 0x5c);
    let k0_xor_ipad = k0.map(|e| e ^ 0x36);

    let mut hasher = T::default();
    hasher.update(&k0_xor_ipad);
    hasher.update(data);
    let rhs = hasher.finish();

    let mut hasher = T::default();
    hasher.update(&k0_xor_opad);
    hasher.update(&rhs);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Md5;

    #[test]
    fn test_hmac() {
        assert_eq!(
            &hmac::<Md5>(b"key", b"The quick brown fox jumps over the lazy dog"),
            &[
                0x80, 0x07, 0x07, 0x13, 0x46, 0x3e, 0x77, 0x49, 0xb9, 0x0c, 0x2d, 0xc2, 0x49, 0x11,
                0xe2, 0x75
            ]
        );
    }
}
