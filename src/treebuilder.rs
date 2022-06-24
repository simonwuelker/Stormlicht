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
    scripting: bool,
    active_formatting_elements
    /// points to the head node
    head: Option<Node>,
    /// points to the last form tag that has not been closed
    form: Option<Node>,
}
