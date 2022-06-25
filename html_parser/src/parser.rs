use crate::tokenizer::Token;

pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

pub struct Parser {
    insertion_modes: Vec<InsertionMode>,
    open_elements: Vec<Node>,
    /// pointer to the head tag
    head: Option<Node>,
    /// pointer to the last open form tag
    form: Option<Node>,
    /// whether or not scripting is enabled
    scripting: bool,
}

impl Parser {
    pub fn new(scripting: bool) -> {
        Self {
            insertion_modes: Vec::new(),
            head: None,
            form: None,
            scripting: scripting,
        }
    }

    fn current_node(&self) -> &Node {
        self.open_elements.first()
    }

    fn consume


}
