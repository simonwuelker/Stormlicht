#[derive(Debug, Clone)]
pub enum Token {
    DOCTYPE(Doctype),
    Tag(TagData),
    Comment(String),
    // TODO: emitting single characters is really inefficient, change this to be a string
    Character(char),
    EOF,
}

#[derive(Debug, Default, Clone)]
pub struct Doctype {
    pub name: Option<String>,
    pub public_ident: Option<String>,
    pub system_ident: Option<String>,
    pub force_quirks: bool,
}

#[derive(Debug, Clone)]
pub struct TagData {
    /// True if the tag is opening (`<tag>`) and false if it's a closing tag (`</tag>`)
    pub opening: bool,

    /// The tag identifier.
    ///
    /// For `<script>`, this would be `"script"` for example.
    pub name: String,

    /// Whether the tag declaration closes itself (`<tag/>`)
    pub self_closing: bool,

    /// A list of tag attributes.
    ///
    /// For example, the tag `<tag foo=bar baz=boo>` has two attributes, `("foo", "bar")` and `("baz", "boo")`.
    pub attributes: Vec<(String, String)>,
}

impl TagData {
    pub fn lookup_attribute<'a>(&'a self, want: &str) -> Option<&'a str> {
        for (key, value) in &self.attributes {
            if key == want {
                return Some(value);
            }
        }
        None
    }

    pub(crate) fn new_attribute(&mut self) {
        self.attributes.push((String::new(), String::new()));
    }

    /// Add a character to the last attribute's name
    pub(crate) fn add_to_attr_name(&mut self, c: char) {
        self.attributes.last_mut().unwrap().0.push(c);
    }

    /// Add a character to the last attribute's value
    pub(crate) fn add_to_attr_value(&mut self, c: char) {
        self.attributes.last_mut().unwrap().1.push(c);
    }

    pub(crate) fn default_open() -> Self {
        Self {
            opening: true,
            name: String::default(),
            self_closing: false,
            attributes: Vec::new(),
        }
    }

    pub(crate) fn default_close() -> Self {
        Self {
            opening: false,
            name: String::default(),
            self_closing: false,
            attributes: Vec::new(),
        }
    }
}
