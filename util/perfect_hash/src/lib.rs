#[cfg(feature = "derive")]
pub use perfect_hash_derive::perfect_set;

// TODO: Find a way to not duplicate this with the definitions in perfect_hash_derive
const fn str_hash(string: &str) -> u32 {
    let b: u32 = 378551;
    let mut a: u32 = 63689;
    let mut hash: u32 = 0;
    let bytes = string.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        hash = hash.wrapping_mul(a).wrapping_add(bytes[i] as u32);
        a = a.wrapping_mul(b);
        i += 1;
    }

    hash
}

const fn int_hash(mut x: u32) -> u32 {
    x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3b);
    x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3b);
    x = (x >> 16) ^ x;
    x
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    // We might to want to store more values here later, like an enum for keywords
    value: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PerfectHashTable<const N: usize> {
    second_level_functions: [u32; N],
    buckets: [Entry; N],
}

impl<const N: usize> PerfectHashTable<N> {
    #[must_use]
    pub const fn new(second_level_functions: [u32; N], buckets: [Entry; N]) -> Self {
        Self {
            second_level_functions,
            buckets,
        }
    }

    #[must_use]
    pub fn compute_hash(&self, item: &str) -> u32 {
        let primary_hash = str_hash(item);
        let secondary_fn = self.second_level_functions[primary_hash as usize % N];
        (int_hash(primary_hash ^ secondary_fn) as usize % N) as u32
    }

    #[must_use]
    pub fn contains(&self, item: &str) -> bool {
        self.buckets[self.compute_hash(item) as usize].value == item
    }
}

impl Entry {
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}
