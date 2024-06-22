//! <https://drafts.csswg.org/css-ui/#cursor>

use crate::{
    css::{
        style::{computed, StyleContext, ToComputedStyle},
        CSSParse, ParseError, Parser,
    },
    static_interned,
};

/// <https://drafts.csswg.org/css-ui/#cursor>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cursor {
    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-default>
    Default,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-none>
    None,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-context-menu>
    ContextMenu,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-help>
    Help,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-pointer>
    Pointer,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-progress>
    Progress,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-wait>
    Wait,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-cell>
    Cell,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-crosshair>
    Crosshair,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-text>
    Text,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-vertical-text>
    VerticalText,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-alias>
    Alias,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-copy>
    Copy,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-move>
    Move,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-no-drop>
    NoDrop,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-not-allowed>
    NotAllowed,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-grab>
    Grab,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-grabbing>
    Grabbing,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-e-resize>
    ResizeEast,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-n-resize>
    ResizeNorth,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-ne-resize>
    ResizeNorthEast,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-nw-resize>
    ResizeNorthWest,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-s-resize>
    ResizeSouth,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-se-resize>
    ResizeSouthEast,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-sw-resize>
    ResizeSouthWest,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-w-resize>
    ResizeWest,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-ew-resize>
    ResizeEastWest,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-ns-resize>
    ResizeNorthSouth,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-nesw-resize>
    ResizeNorthEastSouthWest,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-nwse-resize>
    ResizeNorthWestSouthEast,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-col-resize>
    ResizeColumn,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-row-resize>
    ResizeRow,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-all-scroll>
    AllScroll,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-zoom-in>
    ZoomIn,

    /// <https://drafts.csswg.org/css-ui/#valdef-cursor-zoom-out>
    ZoomOut,
}

impl<'a> CSSParse<'a> for Cursor {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let position = match parser.expect_identifier()? {
            static_interned!("default") => Self::Default,
            static_interned!("none") => Self::None,
            static_interned!("context-menu") => Self::ContextMenu,
            static_interned!("help") => Self::Help,
            static_interned!("pointer") => Self::Pointer,
            static_interned!("progress") => Self::Progress,
            static_interned!("wait") => Self::Wait,
            static_interned!("cell") => Self::Cell,
            static_interned!("crosshair") => Self::Crosshair,
            static_interned!("text") => Self::Text,
            static_interned!("vertical-text") => Self::VerticalText,
            static_interned!("alias") => Self::Alias,
            static_interned!("copy") => Self::Copy,
            static_interned!("move") => Self::Move,
            static_interned!("no-drop") => Self::NoDrop,
            static_interned!("not-allowed") => Self::NotAllowed,
            static_interned!("grab") => Self::Grab,
            static_interned!("grabbing") => Self::Grabbing,
            static_interned!("e-resize") => Self::ResizeEast,
            static_interned!("n-resize") => Self::ResizeNorth,
            static_interned!("ne-resize") => Self::ResizeNorthEast,
            static_interned!("nw-resize") => Self::ResizeNorthWest,
            static_interned!("s-resize") => Self::ResizeSouth,
            static_interned!("se-resize") => Self::ResizeSouthEast,
            static_interned!("sw-resize") => Self::ResizeSouthWest,
            static_interned!("w-resize") => Self::ResizeWest,
            static_interned!("ew-resize") => Self::ResizeEastWest,
            static_interned!("ns-resize") => Self::ResizeNorthSouth,
            static_interned!("nesw-resize") => Self::ResizeNorthEastSouthWest,
            static_interned!("nwse-resize") => Self::ResizeNorthWestSouthEast,
            static_interned!("col-resize") => Self::ResizeColumn,
            static_interned!("row-resize") => Self::ResizeRow,
            static_interned!("all-scroll") => Self::AllScroll,
            static_interned!("zoom-in") => Self::ZoomIn,
            static_interned!("zoom-out") => Self::ZoomOut,
            _ => return Err(ParseError),
        };

        Ok(position)
    }
}

impl ToComputedStyle for Cursor {
    type Computed = computed::Cursor;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;

        *self
    }
}
