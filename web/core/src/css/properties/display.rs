use string_interner::{static_interned, static_str, InternedString};

use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

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
    Flow,
    FlowRoot,
    Table,
    Flex,
    Grid,
    Ruby,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayInsideOutside {
    pub outside: DisplayOutside,
    pub inside: DisplayInside,
    pub list_item_flag: ListItemFlag,
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
pub enum DisplayValue {
    InsideOutside(DisplayInsideOutside),
    Internal(DisplayInternal),
    Box(DisplayBox),
}

impl Default for DisplayValue {
    fn default() -> Self {
        Self::InsideOutside(DisplayInsideOutside {
            outside: DisplayOutside::Inline,
            inside: DisplayInside::Flow,
            list_item_flag: ListItemFlag::No,
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

impl DisplayValue {
    #[must_use]
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            DisplayValue::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                ..
            })
        )
    }

    #[inline]
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.eq(&Self::Box(DisplayBox::None))
    }

    #[inline]
    #[must_use]
    pub fn is_contents(&self) -> bool {
        self.eq(&Self::Box(DisplayBox::Contents))
    }
}

impl From<Short> for DisplayValue {
    fn from(short: Short) -> Self {
        match short {
            Short::None => Self::Box(DisplayBox::None),
            Short::Contents => Self::Box(DisplayBox::Contents),
            Short::Block => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Flow,
                list_item_flag: ListItemFlag::No,
            }),
            Short::FlowRoot => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::FlowRoot,
                list_item_flag: ListItemFlag::No,
            }),
            Short::Inline => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Flow,
                list_item_flag: ListItemFlag::No,
            }),
            Short::InlineBlock => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::FlowRoot,
                list_item_flag: ListItemFlag::No,
            }),
            Short::RunIn => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::RunIn,
                inside: DisplayInside::Flow,
                list_item_flag: ListItemFlag::No,
            }),
            Short::ListItem => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::FlowRoot,
                list_item_flag: ListItemFlag::Yes,
            }),
            Short::Flex => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Flex,
                list_item_flag: ListItemFlag::No,
            }),
            Short::InlineFlex => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Flex,
                list_item_flag: ListItemFlag::No,
            }),
            Short::Grid => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Grid,
                list_item_flag: ListItemFlag::No,
            }),
            Short::InlineGrid => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Grid,
                list_item_flag: ListItemFlag::No,
            }),
            Short::Ruby => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Ruby,
                list_item_flag: ListItemFlag::No,
            }),
            Short::Table => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Block,
                inside: DisplayInside::Table,
                list_item_flag: ListItemFlag::No,
            }),
            Short::InlineTable => Self::InsideOutside(DisplayInsideOutside {
                outside: DisplayOutside::Inline,
                inside: DisplayInside::Table,
                list_item_flag: ListItemFlag::No,
            }),
        }
    }
}

impl<'a> CSSParse<'a> for DisplayValue {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        // A display value always consists of up to three identifiers
        let mut idents = vec![];
        for _ in 0..3 {
            match parser.next_token() {
                Some(Token::Ident(ident)) => idents.push(ident),
                None | Some(Token::Semicolon) => break,
                _ => return Err(ParseError),
            }
            parser.skip_whitespace();
        }

        if idents.len() == 1 {
            let ident = idents[0];
            let short = Short::try_from(ident)?;
            Ok(Self::from(short))
        } else {
            let mut list_item_flag = ListItemFlag::No;
            let mut outside = DisplayOutside::Block;
            let mut inside = DisplayInside::Flow;

            for ident in idents {
                if ident == static_interned!("list-item") {
                    list_item_flag = ListItemFlag::Yes;
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
            if list_item_flag == ListItemFlag::Yes
                && inside != DisplayInside::Flow
                && inside != DisplayInside::FlowRoot
            {
                Err(ParseError)
            } else {
                Ok(Self::InsideOutside(DisplayInsideOutside {
                    outside,
                    inside,
                    list_item_flag,
                }))
            }
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
    #[inline]
    fn from_ident(ident: InternedString) -> Option<Self> {
        match ident {
            static_interned!("flow") => Some(Self::Flow),
            static_interned!("flow-root") => Some(Self::FlowRoot),
            static_interned!("table") => Some(Self::Table),
            static_interned!("flex") => Some(Self::Flex),
            static_interned!("grid") => Some(Self::Grid),
            static_interned!("ruby") => Some(Self::Ruby),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayBox, DisplayValue};
    use crate::css::CSSParse;

    #[test]
    fn parse_box() {
        assert_eq!(
            DisplayValue::parse_from_str("none"),
            Ok(DisplayValue::Box(DisplayBox::None))
        );
        assert_eq!(
            DisplayValue::parse_from_str("contents"),
            Ok(DisplayValue::Box(DisplayBox::Contents))
        );
    }
}
