use crate::{
    css::{
        style::{computed, StyleContext, ToComputedStyle},
        syntax::Token,
        CSSParse, ParseError, Parser,
    },
    static_interned, InternedString,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListItemFlag {
    Yes,
    No,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayOutside {
    Block,
    Inline,
    RunIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayInside {
    Flow { list_item_flag: ListItemFlag },
    FlowRoot { list_item_flag: ListItemFlag },
    Table,
    Flex,
    Grid,
    Ruby,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayInsideOutside {
    pub outside: DisplayOutside,
    pub inside: DisplayInside,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayInternal {
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
    TableColumnGroup,
    TableColumn,
    TableCaption,
    RubyBase,
    RubyText,
    RubyBaseContainer,
    RubyTextContainer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayBox {
    None,
    Contents,
}

/// <https://drafts.csswg.org/css-display/#the-display-properties>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    InsideOutside(DisplayInsideOutside),
    Internal(DisplayInternal),
    Box(DisplayBox),
}

impl Default for Display {
    fn default() -> Self {
        Self::InsideOutside(DisplayInsideOutside {
            outside: DisplayOutside::Inline,
            inside: DisplayInside::Flow {
                list_item_flag: ListItemFlag::No,
            },
        })
    }
}

/// Different ways of specifying a display property with just a single
/// keyword, eg `display: none;`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Short {
    None,
    Contents,
    Block,
    FlowRoot,
    Inline,
    InlineBlock,
    RunIn,
    ListItem,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Ruby,
    Table,
    InlineTable,
}

impl TryFrom<InternedString> for Short {
    type Error = ParseError;

    fn try_from(value: InternedString) -> Result<Self, Self::Error> {
        match value {
            static_interned!("none") => Ok(Self::None),
            static_interned!("contents") => Ok(Self::Contents),
            static_interned!("block") => Ok(Self::Block),
            static_interned!("flow-root") => Ok(Self::FlowRoot),
            static_interned!("inline") => Ok(Self::Inline),
            static_interned!("inline-block") => Ok(Self::InlineBlock),
            static_interned!("run-in") => Ok(Self::RunIn),
            static_interned!("list-item") => Ok(Self::ListItem),
            static_interned!("flex") => Ok(Self::Flex),
            static_interned!("inline-flex") => Ok(Self::InlineFlex),
            static_interned!("grid") => Ok(Self::Grid),
            static_interned!("inline-grid") => Ok(Self::InlineGrid),
            static_interned!("ruby") => Ok(Self::Ruby),
            static_interned!("table") => Ok(Self::Table),
            static_interned!("inline-table") => Ok(Self::InlineTable),
            _ => Err(ParseError),
        }
    }
}

impl Display {
    #[inline]
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::Box(DisplayBox::None))
    }

    #[inline]
    #[must_use]
    pub const fn is_contents(&self) -> bool {
        matches!(self, Self::Box(DisplayBox::Contents))
    }
}

impl From<Short> for Display {
    fn from(short: Short) -> Self {
        match short {
            Short::None => Self::Box(DisplayBox::None),
            Short::Contents => Self::Box(DisplayBox::Contents),
            Short::Block => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Flow {
                    list_item_flag: ListItemFlag::No,
                },
            }),
            Short::FlowRoot => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::FlowRoot {
                    list_item_flag: ListItemFlag::No,
                },
            }),
            Short::Inline => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Flow {
                    list_item_flag: ListItemFlag::No,
                },
            }),
            Short::InlineBlock => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::FlowRoot {
                    list_item_flag: ListItemFlag::No,
                },
            }),
            Short::RunIn => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::RunIn,
                inside: DisplayInside::Flow {
                    list_item_flag: ListItemFlag::No,
                },
            }),
            Short::ListItem => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::FlowRoot {
                    list_item_flag: ListItemFlag::Yes,
                },
            }),
            Short::Flex => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Flex,
            }),
            Short::InlineFlex => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Flex,
            }),
            Short::Grid => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Grid,
            }),
            Short::InlineGrid => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Grid,
            }),
            Short::Ruby => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Ruby,
            }),
            Short::Table => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Table,
            }),
            Short::InlineTable => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Table,
            }),
        }
    }
}

impl<'a> CSSParse<'a> for Display {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        // A display value always consists of up to three identifiers
        let mut idents = vec![];
        for _ in 0..3 {
            match parser.peek_token_ignoring_whitespace(0) {
                Some(Token::Ident(ident)) => {
                    let ident = *ident;
                    let _ = parser.next_token_ignoring_whitespace();
                    idents.push(ident)
                },
                Some(Token::Semicolon | Token::CurlyBraceClose) => break,
                _ => return Err(ParseError),
            }
        }

        if idents.len() == 1 {
            let ident = idents[0];
            let short = Short::try_from(ident)?;
            Ok(Self::from(short))
        } else {
            let mut has_list_item_flag = false;
            let mut outside = DisplayOutside::Block;
            let mut inside = DisplayInside::Flow {
                list_item_flag: ListItemFlag::No,
            };

            for ident in idents {
                if ident == static_interned!("list-item") {
                    has_list_item_flag = true;
                } else if let Some(display_outside) = DisplayOutside::from_ident(ident) {
                    outside = display_outside;
                } else if let Some(display_inside) = DisplayInside::from_ident(ident) {
                    inside = display_inside;
                } else {
                    return Err(ParseError);
                }
            }

            // Only "flow" and "flow-root" are allowed for inside
            // if the list_item_flag is set
            if has_list_item_flag {
                match &mut inside {
                    DisplayInside::Flow { list_item_flag }
                    | DisplayInside::FlowRoot { list_item_flag } => {
                        *list_item_flag = ListItemFlag::Yes
                    },
                    _ => return Err(ParseError),
                }
            }
            let display = Self::InsideOutside(DisplayInsideOutside { outside, inside });

            Ok(display)
        }
    }
}

impl DisplayOutside {
    #[inline]
    fn from_ident(ident: InternedString) -> Option<Self> {
        match ident {
            static_interned!("block") => Some(Self::Block),
            static_interned!("inline") => Some(Self::Inline),
            static_interned!("run-in") => Some(Self::RunIn),
            _ => None,
        }
    }
}

impl DisplayInside {
    #[must_use]
    pub const fn has_list_item_flag(&self) -> bool {
        matches!(
            self,
            Self::Flow {
                list_item_flag: ListItemFlag::Yes
            } | Self::FlowRoot {
                list_item_flag: ListItemFlag::Yes
            }
        )
    }

    #[inline]
    fn from_ident(ident: InternedString) -> Option<Self> {
        match ident {
            static_interned!("flow") => Some(Self::Flow {
                list_item_flag: ListItemFlag::No,
            }),
            static_interned!("flow-root") => Some(Self::FlowRoot {
                list_item_flag: ListItemFlag::No,
            }),
            static_interned!("table") => Some(Self::Table),
            static_interned!("flex") => Some(Self::Flex),
            static_interned!("grid") => Some(Self::Grid),
            static_interned!("ruby") => Some(Self::Ruby),
            _ => None,
        }
    }
}

impl ToComputedStyle for Display {
    type Computed = computed::Display;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;

        *self
    }
}
