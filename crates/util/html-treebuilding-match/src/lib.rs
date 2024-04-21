#![allow(dead_code)] // Clippy seems to have issues with the quote! macro

use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Block, Expr, Ident, Pat, Result, Token,
};

struct MatchArm {
    pattern: MatchPattern,
    guard: Option<Expr>,
    body: Block,
}

enum MatchPattern {
    Traditional(Pat),
    Custom(Vec<HtmlTag>),
}

struct HtmlTag {
    is_opening: bool,
    ident: Ident,
}

struct StateMachine {
    match_on: Expr,
    arms: Vec<MatchArm>,
}

impl Parse for StateMachine {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let match_on: Expr = input.parse()?;
        let _: Token![,] = input.parse()?;

        let mut arms = vec![];
        while !input.is_empty() {
            let arm: MatchArm = input.parse()?;
            let _: Token![,] = input.parse()?;
            arms.push(arm);
        }

        let state_matchine = Self { match_on, arms };
        Ok(state_matchine)
    }
}

impl Parse for MatchArm {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let pattern = input.parse()?;

        let guard = if input.parse::<Option<Token![if]>>()?.is_some() {
            Some(input.parse()?)
        } else {
            None
        };

        let _: Token![=>] = input.parse()?;
        let body = input.parse()?;

        let match_arm = Self {
            pattern,
            guard,
            body,
        };
        Ok(match_arm)
    }
}

impl Parse for MatchPattern {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead1 = input.lookahead1();

        if lookahead1.peek(Token![<]) {
            // This is a custom html arm
            let tag = input.parse()?;
            let mut tags = vec![tag];
            while input.parse::<Option<Token![|]>>()?.is_some() {
                let tag = input.parse()?;
                tags.push(tag);
            }

            Ok(Self::Custom(tags))
        } else {
            // This is a "regular rust" match arm
            let pat = Pat::parse_multi(input)?;
            let arm = Self::Traditional(pat);

            Ok(arm)
        }
    }
}

impl Parse for HtmlTag {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let _: Token![<] = input.parse()?;

        let slash: Option<Token![/]> = input.parse()?;
        let ident: Ident = input.parse()?;
        let _: Token![>] = input.parse()?;

        let tag = Self {
            is_opening: slash.is_none(),
            ident,
        };

        Ok(tag)
    }
}

impl ToTokens for StateMachine {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { match_on, arms } = self;

        quote!(
            match #match_on {
                #(#arms)*
            }
        )
        .to_tokens(tokens)
    }
}

impl ToTokens for MatchArm {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            pattern,
            guard,
            body,
        } = self;

        if let Some(guard) = guard {
            quote!(
                #pattern if #guard => #body,
            )
            .to_tokens(tokens)
        } else {
            quote!(
                #pattern => #body,
            )
            .to_tokens(tokens)
        }
    }
}

impl ToTokens for MatchPattern {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Custom(tags) => {
                let mut opening_idents = vec![];
                let mut closing_idents = vec![];

                for tag in tags {
                    if tag.is_opening {
                        opening_idents.push(tag.ident.to_string());
                    } else {
                        closing_idents.push(tag.ident.to_string());
                    }
                }

                let opening_condition = quote!(#(| static_interned!(#opening_idents))*);
                let closing_condition = quote!(#(| static_interned!(#closing_idents))*);

                let token_stream = if opening_idents.len() != 0 {
                    if closing_idents.len() != 0 {
                        quote!(
                            Token::StartTag(tag @ TagData { name: #opening_condition, ..}) | Token::EndTag(tag @ TagData { name: #closing_condition, ..})
                        )
                    } else {
                        quote!(
                            Token::StartTag(tag @ TagData { name: #opening_condition, ..})
                        )
                    }
                } else if closing_idents.len() != 0 {
                    quote!(
                        Token::EndTag(tag @ TagData { name: #closing_condition, ..})
                    )
                } else {
                    unreachable!()
                };

                token_stream.to_tokens(tokens);
            },
            Self::Traditional(pattern) => pattern.to_tokens(tokens),
        }
    }
}

/// Allows for slightly nicer definition of the html treebuilding state machine
///
/// HTML Treebuilding is a state machine that transitions based on input tokens encountered.
/// This leads to huge match blocks. (see `treebuilding/parser.rs` in `web`)
///
/// Without this macro, they look something like this:
/// ```ignore
/// match token {
///     Token::StartTag(tag) if tag.name == "html" => { ... }
///     Token::StartTag(tag) if tag.name == "base" | tag.name == "basefont" => { ... }
/// }
/// ```
/// With the help of this macro they can instead be written as
/// ```ignore
/// html_treebuilding_match!(token,
///     <html> = { ... }
///     <base> | <basefont> => { ... }
/// )
/// ```
///
/// Note that the non-tag tokens (`Token::Character` for example) can still be matched on as usual.
/// In general, everything that is legal in a regular `match` is also legal in `html_treebuilding_match`.
///
/// Within every match arm, a `tag` variable is available containing the tag data in case its needed.
#[proc_macro]
pub fn html_treebuilding_match(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let state_machine = parse_macro_input!(input as StateMachine);
    state_machine.to_token_stream().into()
}
