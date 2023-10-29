use sl_std::big_num::BigNum;

#[derive(Clone, Debug)]
pub enum Integer {
    Small(usize),
    Big(BigNum),
}

impl TryFrom<Integer> for usize {
    type Error = ();

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        match value {
            Integer::Small(int) => Ok(int),
            Integer::Big(_) => Err(()),
        }
    }
}

impl From<Integer> for BigNum {
    fn from(value: Integer) -> Self {
        match value {
            Integer::Small(int) => BigNum::from_be_bytes(&int.to_be_bytes()),
            Integer::Big(bigint) => bigint,
        }
    }
}

impl Integer {
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        const BYTES_IN_USIZE: usize = (usize::BITS / 8) as usize;

        if bytes.len() <= BYTES_IN_USIZE {
            let mut buffer = [0; BYTES_IN_USIZE];
            buffer[BYTES_IN_USIZE - bytes.len()..].copy_from_slice(bytes);
            Self::Small(usize::from_be_bytes(buffer))
        } else {
            Self::Big(BigNum::from_be_bytes(bytes))
        }
    }
}
