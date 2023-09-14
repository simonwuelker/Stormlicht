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
    ($value: tt) => {
        $crate::InternedString::Static(static_str!($value))
    };
}

// TODO: might want to figure out a nice way to define this outside of this crate later
// That way, users that don't want to write a browser engine could also use it :)
#[rustfmt::skip]
perfect_set!(
    const STATIC_SET = [
        "",

        // html tags
        "a",
        "address",
        "area",
        "article",
        "applet",
        "aside",
        "b",
        "base",
        "basefont",
        "bgsound",
        "big",
        "blockquote",
        "body",
        "br",
        "button",
        "caption",
        "center",
        "code",
        "col",
        "colgroup",
        "dd",
        "details",
        "dialog",
        "dir",
        "div",
        "dl",
        "dt",
        "em",
        "embed",
        "fieldset",
        "figcaption",
        "figure",
        "font",
        "footer",
        "form",
        "frame",
        "frameset",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "head",
        "header",
        "hgroup",
        "hr",
        "html",
        "i",
        "iframe",
        "image",
        "img",
        "input",
        "keygen",
        "li",
        "link",
        "listing",
        "main",
        "marquee",
        "math",
        "menu",
        "meta",
        "nav",
        "nobr",
        "noembed",
        "noframes",
        "noscript",
        "object",
        "ol",
        "optgroup",
        "option",
        "p",
        "param",
        "plaintext",
        "pre",
        "rb",
        "rtc",
        "s",
        "script",
        "section",
        "select",
        "small",
        "source",
        "strike",
        "strong",
        "style",
        "summary",
        "svg",
        "table",
        "tbody",
        "td",
        "template",
        "textarea",
        "tfoot",
        "th",
        "thead",
        "title",
        "tr",
        "track",
        "tt",
        "u",
        "ul",
        "wbr",
        "xmp",

        // html attributes
        "is",
        "type",
        "id",
        "class",

        // CSS terms
        "after",
        "auto",
        "background-color",
        "before",
        "block",
        "color",
        "contents",
        "display",
        "first-line",
        "first-letter",
        "flex",
        "flow",
        "flow-root",
        "grid",
        "height",
        "hidden",
        "important",
        "inherit",
        "inline",
        "inline-block",
        "inline-flex",
        "inline-grid",
        "inline-table",
        "list-item",
        "margin-bottom",
        "margin-left",
        "margin-right",
        "margin-top",
        "none",
        "padding-bottom",
        "padding-left",
        "padding-right",
        "padding-top",
        "rgb",
        "rgba",
        "ruby",
        "run-in",
        "transparent",
        "width",

        // CSS Colors
        "aliceblue",
        "antiquewhite",
        "aqua",
        "aquamarine",
        "azure",
        "beige",
        "bisque",
        "black",
        "blanchedalmond",
        "blue",
        "blueviolet",
        "brown",
        "burlywood",
        "cadetblue",
        "chartreuse",
        "chocolate",
        "coral",
        "cornflowerblue",
        "cornsilk",
        "crimson",
        "cyan",
        "darkblue",
        "darkcyan",
        "darkgoldenrod",
        "darkgray",
        "darkgreen",
        "darkgrey",
        "darkkhaki",
        "darkmagenta",
        "darkolivegreen",
        "darkorange",
        "darkorchid",
        "darkred",
        "darksalmon",
        "darkseagreen",
        "darkslateblue",
        "darkslategray",
        "darkslategrey",
        "darkturquoise",
        "darkviolet",
        "deeppink",
        "deepskyblue",
        "dimgray",
        "dimgrey",
        "dodgerblue",
        "firebrick",
        "floralwhite",
        "forestgreen",
        "fuchsia",
        "gainsboro",
        "ghostwhite",
        "gold",
        "goldenrod",
        "gray",
        "green",
        "greenyellow",
        "grey",
        "honeydew",
        "hotpink",
        "indianred",
        "indigo",
        "ivory",
        "khaki",
        "lavender",
        "lavenderblush",
        "lawngreen",
        "lemonchiffon",
        "lightblue",
        "lightcoral",
        "lightcyan",
        "lightgoldenrodyellow",
        "lightgray",
        "lightgreen",
        "lightgrey",
        "lightpink",
        "lightsalmon",
        "lightseagreen",
        "lightskyblue",
        "lightslategray",
        "lightslategrey",
        "lightsteelblue",
        "lightyellow",
        "lime",
        "limegreen",
        "linen",
        "magenta",
        "maroon",
        "mediumaquamarine",
        "mediumblue",
        "mediumorchid",
        "mediumpurple",
        "mediumseagreeen",
        "mediumslateblue",
        "mediumspringgreen",
        "mediumturquoise",
        "mediumvioletred",
        "midnightblue",
        "mintcream",
        "mistyrose",
        "moccasin",
        "navajowhite",
        "navy",
        "oldlace",
        "olive",
        "olivedrab",
        "orange",
        "orangered",
        "orchid",
        "palegoldenrod",
        "palegreen",
        "paleturquoise",
        "palevioletred",
        "papayawhip",
        "peachpuff",
        "peru",
        "pink",
        "plum",
        "powderblue",
        "purple",
        "rebeccapurple",
        "red",
        "rosybrown",
        "royalblue",
        "saddlebrown",
        "salmon",
        "sandybrown",
        "seagreen",
        "seashell",
        "sienna",
        "silver",
        "skyblue",
        "slateblue",
        "slategray",
        "slategrey",
        "snow",
        "springgreen",
        "steelblue",
        "tan",
        "teal",
        "thistle",
        "tomato",
        "turquoise",
        "violet",
        "wheat",
        "white",
        "whitesmoke",
        "yellow",
        "yellowgreen",

        // CSS Length Units
        "rem",
        "ex",
        "rex",
        "cap",
        "rcap",
        "ch",
        "rch",
        "ic",
        "ric",
        "lh",
        "rlh",
        "vw",
        "svw",
        "lvw",
        "dvw",
        "vh",
        "svh",
        "lvh",
        "dvh",
        "vi",
        "svi",
        "lvi",
        "dvi",
        "vb",
        "svb",
        "lvb",
        "dvb",
        "vmin",
        "svmin",
        "lvmin",
        "dvmin",
        "vmax",
        "svmax",
        "lvmax",
        "dvmax",
        "cm",
        "mm",
        "q",
        "in",
        "pc",
        "pt",
        "px",
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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl Default for InternedString {
    fn default() -> Self {
        static_interned!("")
    }
}

impl fmt::Debug for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternedString::Static(symbol) => {
                write!(f, "{:?}_s", STATIC_SET.lookup(*symbol))
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

                write!(f, "{string:?}_d")
            },
        }
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

impl From<String> for InternedString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for InternedString {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
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
