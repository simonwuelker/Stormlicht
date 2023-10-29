use string_interner::InternedString;

#[derive(Debug, Clone)]
pub enum Token {
    DOCTYPE(Doctype),
    Tag(TagData),
    Comment(String),
    // TODO: emitting single characters is really inefficient, change this to be a string
    Character(char),
    EOF,
}

#[derive(Debug, Clone, Default)]
pub struct CurrentToken {
    current_token: Option<TokenBuilder>,
}

#[derive(Debug, Clone)]
pub enum TokenBuilder {
    DOCTYPE(DocTypeBuilder),
    Tag(TagBuilder),
    Comment(String),
}

#[derive(Debug, Clone, Default)]
pub struct DocTypeBuilder {
    pub name: Option<String>,
    pub public_ident: Option<String>,
    pub system_ident: Option<String>,
    pub force_quirks: bool,
}

#[derive(Debug, Clone)]
pub struct TagBuilder {
    pub opening: bool,
    pub name: String,
    pub self_closing: bool,
    pub attributes: Vec<(String, String)>,
}

impl CurrentToken {
    // FIXME: this is an ugly escape hatch, we shouldn't need this
    pub(crate) fn inner(&self) -> Option<&TokenBuilder> {
        self.current_token.as_ref()
    }

    pub fn create_start_tag(&mut self) {
        self.current_token = Some(TokenBuilder::Tag(TagBuilder::new_opening()))
    }

    pub fn create_end_tag(&mut self) {
        self.current_token = Some(TokenBuilder::Tag(TagBuilder::new_closing()))
    }

    pub fn create_comment(&mut self) {
        self.current_token = Some(TokenBuilder::Comment(String::new()))
    }

    pub fn create_doctype(&mut self) {
        self.current_token = Some(TokenBuilder::DOCTYPE(DocTypeBuilder::default()))
    }

    pub fn set_force_quirks(&mut self) {
        if let Some(TokenBuilder::DOCTYPE(DocTypeBuilder {
            ref mut force_quirks,
            ..
        })) = self.current_token
        {
            *force_quirks = true;
        }
    }

    pub fn append_to_doctype_name(&mut self, c: char) {
        match self.current_token {
            Some(TokenBuilder::DOCTYPE(DocTypeBuilder {
                name: Some(ref mut name_str),
                ..
            })) => name_str.push(c),
            Some(TokenBuilder::DOCTYPE(DocTypeBuilder { ref mut name, .. })) => {
                *name = Some(c.to_string())
            },
            _ => {},
        }
    }

    pub fn append_to_doctype_public_ident(&mut self, c: char) {
        if let Some(TokenBuilder::DOCTYPE(DocTypeBuilder {
            public_ident: Some(ref mut public_ident_str),
            ..
        })) = self.current_token
        {
            public_ident_str.push(c);
        }
    }

    pub fn init_doctype_public_ident(&mut self) {
        if let Some(TokenBuilder::DOCTYPE(DocTypeBuilder {
            ref mut public_ident,
            ..
        })) = self.current_token
        {
            *public_ident = Some(String::new());
        }
    }

    pub fn append_to_doctype_system_ident(&mut self, c: char) {
        match self.current_token {
            Some(TokenBuilder::DOCTYPE(DocTypeBuilder {
                system_ident: Some(ref mut system_ident_str),
                ..
            })) => system_ident_str.push(c),
            Some(TokenBuilder::DOCTYPE(DocTypeBuilder {
                ref mut system_ident,
                ..
            })) => *system_ident = Some(c.to_string()),
            _ => {},
        }
    }

    pub fn init_doctype_system_ident(&mut self) {
        if let Some(TokenBuilder::DOCTYPE(DocTypeBuilder {
            ref mut system_ident,
            ..
        })) = self.current_token
        {
            *system_ident = Some(String::new());
        }
    }

    pub fn append_to_tag_name(&mut self, c: char) {
        if let Some(TokenBuilder::Tag(TagBuilder { ref mut name, .. })) = self.current_token {
            name.push(c);
        }
    }

    pub fn append_to_comment(&mut self, c: char) {
        if let Some(TokenBuilder::Comment(ref mut comment)) = self.current_token {
            comment.push(c);
        }
    }

    pub fn set_self_closing(&mut self) {
        if let Some(TokenBuilder::Tag(TagBuilder {
            ref mut self_closing,
            ..
        })) = self.current_token
        {
            *self_closing = true;
        }
    }

    pub fn start_new_attribute(&mut self) {
        if let Some(TokenBuilder::Tag(TagBuilder {
            opening: true,
            ref mut attributes,
            ..
        })) = self.current_token
        {
            attributes.push((String::new(), String::new()));
        }
    }

    pub fn append_to_attribute_name(&mut self, c: char) {
        if let Some(TokenBuilder::Tag(TagBuilder {
            opening: true,
            ref mut attributes,
            ..
        })) = self.current_token
        {
            if let Some(last_attribute) = attributes.last_mut() {
                last_attribute.0.push(c);
            }
        }
    }

    pub fn append_to_attribute_value(&mut self, c: char) {
        if let Some(TokenBuilder::Tag(TagBuilder {
            opening: true,
            ref mut attributes,
            ..
        })) = self.current_token
        {
            if let Some(last_attribute) = attributes.last_mut() {
                last_attribute.1.push(c);
            }
        }
    }

    pub fn build(&mut self) -> Token {
        match self.current_token.take() {
            Some(TokenBuilder::DOCTYPE(d)) => Token::DOCTYPE(d.build()),
            Some(TokenBuilder::Comment(c)) => Token::Comment(c),
            Some(TokenBuilder::Tag(t)) => Token::Tag(t.build()),
            None => {
                panic!("Trying to emit a token but no token has been constructed")
            },
        }
    }
}

impl TagBuilder {
    pub(crate) fn new_opening() -> Self {
        Self {
            opening: true,
            name: String::default(),
            self_closing: false,
            attributes: Vec::new(),
        }
    }

    pub(crate) fn new_closing() -> Self {
        Self {
            opening: false,
            name: String::default(),
            self_closing: false,
            attributes: Vec::new(),
        }
    }

    pub fn build(self) -> TagData {
        TagData {
            opening: self.opening,
            name: InternedString::new(self.name),
            self_closing: self.self_closing,
            attributes: self
                .attributes
                .into_iter()
                .map(|(k, v)| (InternedString::new(k), InternedString::new(v)))
                .collect(),
        }
    }
}

impl DocTypeBuilder {
    pub fn build(self) -> Doctype {
        Doctype {
            name: self.name.map(InternedString::new),
            public_ident: self.public_ident.map(InternedString::new),
            system_ident: self.system_ident.map(InternedString::new),
            force_quirks: self.force_quirks,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct Doctype {
    pub name: Option<InternedString>,
    pub public_ident: Option<InternedString>,
    pub system_ident: Option<InternedString>,
    pub force_quirks: bool,
}

#[derive(Debug, Clone)]
pub struct TagData {
    /// True if the tag is opening (`<tag>`) and false if it's a closing tag (`</tag>`)
    pub opening: bool,

    /// The tag identifier.
    ///
    /// For `<script>`, this would be `"script"` for example.
    pub name: InternedString,

    /// Whether the tag declaration closes itself (`<tag/>`)
    pub self_closing: bool,

    /// A list of tag attributes.
    ///
    /// For example, the tag `<tag foo=bar baz=boo>` has two attributes, `("foo", "bar")` and `("baz", "boo")`.
    pub attributes: Vec<(InternedString, InternedString)>,
}

impl TagData {
    pub fn lookup_attribute(&self, want: InternedString) -> Option<InternedString> {
        for (key, value) in &self.attributes {
            if *key == want {
                return Some(*value);
            }
        }
        None
    }

    #[inline]
    #[must_use]
    pub fn attributes(&self) -> &[(InternedString, InternedString)] {
        &self.attributes
    }
}
