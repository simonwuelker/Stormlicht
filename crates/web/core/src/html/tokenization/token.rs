use crate::{static_interned, InternedString};

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
    Doctype(DocTypeBuilder),
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
        self.current_token = Some(TokenBuilder::Doctype(DocTypeBuilder::default()))
    }

    pub fn set_force_quirks(&mut self) {
        if let Some(TokenBuilder::Doctype(DocTypeBuilder {
            ref mut force_quirks,
            ..
        })) = self.current_token
        {
            *force_quirks = true;
        }
    }

    pub fn append_to_doctype_name(&mut self, c: char) {
        match self.current_token {
            Some(TokenBuilder::Doctype(DocTypeBuilder {
                name: Some(ref mut name_str),
                ..
            })) => name_str.push(c),
            Some(TokenBuilder::Doctype(DocTypeBuilder { ref mut name, .. })) => {
                *name = Some(c.to_string())
            },
            _ => {},
        }
    }

    pub fn append_to_doctype_public_ident(&mut self, c: char) {
        if let Some(TokenBuilder::Doctype(DocTypeBuilder {
            public_ident: Some(ref mut public_ident_str),
            ..
        })) = self.current_token
        {
            public_ident_str.push(c);
        }
    }

    pub fn init_doctype_public_ident(&mut self) {
        if let Some(TokenBuilder::Doctype(DocTypeBuilder {
            ref mut public_ident,
            ..
        })) = self.current_token
        {
            *public_ident = Some(String::new());
        }
    }

    pub fn append_to_doctype_system_ident(&mut self, c: char) {
        match self.current_token {
            Some(TokenBuilder::Doctype(DocTypeBuilder {
                system_ident: Some(ref mut system_ident_str),
                ..
            })) => system_ident_str.push(c),
            Some(TokenBuilder::Doctype(DocTypeBuilder {
                ref mut system_ident,
                ..
            })) => *system_ident = Some(c.to_string()),
            _ => {},
        }
    }

    pub fn init_doctype_system_ident(&mut self) {
        if let Some(TokenBuilder::Doctype(DocTypeBuilder {
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
            Some(TokenBuilder::Doctype(d)) => Token::DOCTYPE(d.build()),
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

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-mathml-attributes>
    pub fn adjust_mathml_attributes(&mut self) {
        for (key, _) in self.attributes.iter_mut() {
            if *key == static_interned!("definitionurl") {
                *key = static_interned!("definitionUrl");
                break; // attribute names are unique
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes>
    pub fn adjust_foreign_attributes(&mut self) {
        _ = self;
        // FIXME: implement this!
        //        This requires "namespaced attributes"
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjust-svg-attributes>
    pub fn adjust_svg_attributes(&mut self) {
        for (key, _) in self.attributes.iter_mut() {
            let adjusted_key = match key {
                static_interned!("attributename") => static_interned!("attributeName"),
                static_interned!("attributetype") => static_interned!("attributeType"),
                static_interned!("basefrequency") => static_interned!("baseFrequency"),
                static_interned!("baseprofile") => static_interned!("baseProfile"),
                static_interned!("calcmode") => static_interned!("calcMode"),
                static_interned!("clippathunits") => static_interned!("clipPathUnits"),
                static_interned!("diffuseconstant") => static_interned!("diffuseConstant"),
                static_interned!("edgemode") => static_interned!("edgeMode"),
                static_interned!("filterunits") => static_interned!("filterUnits"),
                static_interned!("glyphref") => static_interned!("glyphRef"),
                static_interned!("gradienttransform") => static_interned!("gradientTransform"),
                static_interned!("gradientunits") => static_interned!("gradientUnits"),
                static_interned!("kernelmatrix") => static_interned!("kernelMatrix"),
                static_interned!("kernelunitlength") => static_interned!("kernelUnitLength"),
                static_interned!("keypoints") => static_interned!("keyPoints"),
                static_interned!("keysplines") => static_interned!("keySplines"),
                static_interned!("keytimes") => static_interned!("keyTimes"),
                static_interned!("lengthadjust") => static_interned!("lengthAdjust"),
                static_interned!("limitingconeangle") => static_interned!("limitingConeAngle"),
                static_interned!("markerheight") => static_interned!("markerHeight"),
                static_interned!("markerunits") => static_interned!("markerUnits"),
                static_interned!("markerwidth") => static_interned!("markerWidth"),
                static_interned!("maskcontentunits") => static_interned!("maskContentUnits"),
                static_interned!("maskunits") => static_interned!("maskUnits"),
                static_interned!("numoctaves") => static_interned!("numOctaves"),
                static_interned!("pathlength") => static_interned!("pathLength"),
                static_interned!("patterncontentunits") => static_interned!("patternContentUnits"),
                static_interned!("patterntransform") => static_interned!("patternTransform"),
                static_interned!("patternunits") => static_interned!("patternUnits"),
                static_interned!("pointsatx") => static_interned!("pointsAtX"),
                static_interned!("pointsaty") => static_interned!("pointsAtY"),
                static_interned!("pointsatz") => static_interned!("pointsAtZ"),
                static_interned!("preservealpha") => static_interned!("preserveAlpha"),
                static_interned!("preserveaspectratio") => static_interned!("preserveAspectRatio"),
                static_interned!("primitiveunits") => static_interned!("primitiveUnits"),
                static_interned!("refx") => static_interned!("refX"),
                static_interned!("refy") => static_interned!("refY"),
                static_interned!("repeatcount") => static_interned!("repeatCount"),
                static_interned!("repeatdur") => static_interned!("repeatDur"),
                static_interned!("requiredextensions") => static_interned!("requiredExtensions"),
                static_interned!("requiredfeatures") => static_interned!("requiredFeatures"),
                static_interned!("specularconstant") => static_interned!("specularConstant"),
                static_interned!("specularexponent") => static_interned!("specularExponent"),
                static_interned!("spreadmethod") => static_interned!("spreadMethod"),
                static_interned!("startoffset") => static_interned!("startOffset"),
                static_interned!("stddeviation") => static_interned!("stdDeviation"),
                static_interned!("stitchtiles") => static_interned!("stitchTiles"),
                static_interned!("surfacescale") => static_interned!("surfaceScale"),
                static_interned!("systemlanguage") => static_interned!("systemLanguage"),
                static_interned!("tablevalues") => static_interned!("tableValues"),
                static_interned!("targetx") => static_interned!("targetX"),
                static_interned!("targety") => static_interned!("targetY"),
                static_interned!("textlength") => static_interned!("textLength"),
                static_interned!("viewbox") => static_interned!("viewBox"),
                static_interned!("viewtarget") => static_interned!("viewTarget"),
                static_interned!("xchannelselector") => static_interned!("xChannelSelector"),
                static_interned!("ychannelselector") => static_interned!("yChannelSelector"),
                static_interned!("zoomandpan") => static_interned!("zoomAndPan"),
                _ => continue,
            };

            *key = adjusted_key;
        }
    }
}
