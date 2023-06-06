#[cfg(feature = "derive")]
pub use perfect_hash_derive::perfect_set;

// TODO: Find a way to not duplicate this with the definitions in perfect_hash_derive
pub const fn str_hash(string: &str) -> u32 {
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

pub const fn int_hash(mut x: u32) -> u32 {
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

    /// This will be unique for all strings that are stored in the hashmap
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

    #[must_use]
    pub fn try_get(&self, item: &str) -> Option<u32> {
        let hash = self.compute_hash(item);
        if self.buckets[hash as usize].value == item {
            Some(hash)
        } else {
            None
        }
    }

    /// Get a contained string by its hash.
    ///
    /// Notably, we do not verify wether the hash is actually contained in the
    /// map. If it is not, some other string will be returned. This should generally not
    /// be a problem, since the hashes are opaque to the user and aren't ever modified
    /// after creation. (And of course the static hashmap never deallocates entries either).
    #[must_use]
    pub fn lookup(&self, hash: u32) -> &'static str {
        self.buckets[hash as usize].value
    }
}

impl Entry {
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        Self { value }
    }
}
