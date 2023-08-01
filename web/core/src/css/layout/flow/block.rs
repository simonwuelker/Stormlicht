use super::InlineFormattingContext;

/// <https://drafts.csswg.org/css2/#block-formatting>
#[derive(Clone, Debug)]
pub struct BlockFormattingContext {
    contents: Vec<BlockLevelBox>,
}

/// <https://drafts.csswg.org/css2/#block-level-boxes>
#[derive(Clone, Debug)]
pub struct BlockLevelBox {
    contents: BlockContainer,
}

/// <https://drafts.csswg.org/css2/#block-container-box>
#[derive(Clone, Debug)]
pub enum BlockContainer {
    BlockLevelBoxes(Vec<BlockLevelBox>),
    InlineFormattingContext(InlineFormattingContext),
}


impl BlockLevelBox {
    pub fn new(contents: BlockContainer) -> Self {
        Self {
            contents
        }
    }
}

impl BlockFormattingContext {
    pub fn new(contents: Vec<BlockLevelBox>) -> Self {
        Self {
            contents
        }
    }
}