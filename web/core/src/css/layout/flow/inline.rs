/// <https://drafts.csswg.org/css2/#inline-level-boxes>
#[derive(Clone, Debug)]
pub enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(String),
}

/// <https://drafts.csswg.org/css2/#inline-box>
#[derive(Clone, Debug)]
pub struct InlineBox;

/// <https://drafts.csswg.org/css2/#inline-formatting>
#[derive(Clone, Debug)]
pub struct InlineFormattingContext {
    contents: Vec<InlineLevelBox>,
}
