use url::URL;

use crate::tree::Rule;

/// <https://drafts.csswg.org/cssom-1/#css-style-sheets>
/// Note that since we don't support scripting, the vast majority of the spec
/// is not implemented (and doesn't need to be, for the time being)
#[derive(Clone, Debug)]
pub struct Stylesheet<'a> {
    location: Option<URL>,
    pub(crate) rules: Vec<Rule<'a>>,
}

impl<'a> Stylesheet<'a> {
    pub fn new(location: Option<URL>) -> Self {
        Self {
            location,
            rules: vec![],
        }
    }

    pub fn location(&self) -> Option<&URL> {
        self.location.as_ref()
    }

    pub fn r#type(&self) -> &'static str {
        "text/css"
    }
}
