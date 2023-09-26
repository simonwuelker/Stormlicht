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
