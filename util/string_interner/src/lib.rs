#![feature(lazy_cell, hasher_prefixfree_extras)]
#![feature(const_trait_impl, const_for, const_mut_refs, array_chunks)]

use std::{
    collections::HashMap,
    fmt,
    sync::{LazyLock, Mutex},
};

use perfect_hash::{perfect_set, str_hash};

#[macro_export]
macro_rules! static_interned {
    ($value: expr) => {
        ::$crate::InternedString::Static(static_str!($value))
    };
}

// TODO: might want to figure out a nice way to define this outside of this crate later
// That way, users that don't want to write a browser engine could also use it :)
perfect_set!(
    const STATIC_SET = [
        // html tags
        "html",
        "head",
        "body",

        // CSS terms
        "color",
        "background-color",
    ];
);

static INTERNER: LazyLock<Mutex<StringInterner>> =
    LazyLock::new(|| Mutex::new(StringInterner::new()));

/// Like a [String] that is `Copy` and supports comparison in `O(1)`.
///
/// [InternedStrings](InternedString) hold references into an [StringInterner],
/// which actually stores the strings.
/// This has a few implications:
/// * [InternedStrings](InternedString) are immutable
/// * No deallocation (for now)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InternedString {
    Static(u32),
    Dynamic(u32),
}

// https://github.com/servo/servo/issues/2217
#[derive(Debug)]
pub struct StringInterner {
    internal_map: HashMap<String, u32>,
}

impl StringInterner {
    #[must_use]
    fn new() -> Self {
        Self {
            internal_map: HashMap::new(),
        }
    }

    fn get_or_insert(&mut self, value: String) -> InternedString {
        if let Some(symbol) = STATIC_SET.try_get(&value) {
            return InternedString::Static(symbol);
        }

        let symbol = self.internal_map.get(&value).copied().unwrap_or_else(|| {
            let hash = str_hash(&value);
            self.internal_map.insert(value, hash);
            hash
        });

        InternedString::Dynamic(symbol)
    }
}

impl InternedString {
    pub fn new(from: String) -> Self {
        INTERNER
            .lock()
            .expect("String interner was poisoned")
            .get_or_insert(from)
    }
}

impl fmt::Display for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternedString::Static(symbol) => {
                write!(f, "{}", STATIC_SET.lookup(*symbol))
            },
            InternedString::Dynamic(symbol) => {
                let map = &INTERNER
                    .lock()
                    .expect("String interner was poisoned")
                    .internal_map;

                let string = map
                    .iter()
                    .find(|(_, &v)| v == *symbol)
                    .expect("InternedString not present in Interner")
                    .0;

                write!(f, "{string}")
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::InternedString;

    #[test]
    fn lookup_strings() {
        let foo = InternedString::new("Foo".to_string());
        assert_eq!(&foo.to_string(), "Foo");

        let backtrace = InternedString::new("Bar".to_string());
        assert_eq!(&backtrace.to_string(), "Bar");
    }
}
