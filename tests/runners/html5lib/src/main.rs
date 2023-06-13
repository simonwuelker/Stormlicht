//! A runner for [html5 tokenizer tests](https://github.com/html5lib/html5lib-tests)

mod escape;

use cli::CommandLineArgumentParser;
use core::html::tokenization::{Token, Tokenizer, TokenizerState};

use crate::escape::{unescape_str, unicode_escape};

#[derive(Debug, Default, CommandLineArgumentParser)]
struct ArgumentParser {
    #[argument(
        short_name = 's',
        long_name = "state",
        description = "Initial tokenizer state"
    )]
    initial_state: String,

    #[argument(
        short_name = 'i',
        long_name = "input",
        description = "HTML source that should be tokenized"
    )]
    input: String,

    #[argument(
        may_be_omitted,
        short_name = 'l',
        long_name = "last-start-tag",
        description = "The name of the current test case"
    )]
    last_start_tag: Option<String>,

    #[argument(
        flag,
        short_name = 'v',
        long_name = "version",
        description = "Show package version"
    )]
    version: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Error {
    InvalidInitialState,
}

fn main() -> Result<(), Error> {
    let args = match ArgumentParser::parse() {
        Ok(args) => args,
        Err(_) => {
            println!("{}", ArgumentParser::help());
            return Ok(());
        },
    };

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
    } else {
        let last_start_tag = args
            .last_start_tag
            .clone()
            .map(|t| t[1..t.len() - 1].to_string());

        let source =
            unescape_str(&args.input[1..args.input.len() - 1]).expect("Invalid input text");

        // our commandline parser doesnt handle quotes very well...
        let initial_state =
            parse_initial_state(&args.initial_state[1..args.initial_state.len() - 1])?;

        let mut tokenizer = Tokenizer::new(&source);
        tokenizer.switch_to(initial_state);
        tokenizer.set_last_start_tag(last_start_tag);

        let mut serialized_tokens = vec![];
        while let Some(token) = tokenizer.next() {
            if serialize_token(token, &mut tokenizer, &mut serialized_tokens) {
                break;
            }
        }

        let result = format!("[{}]", serialized_tokens.join(","));
        println!("{result}");
    }
    Ok(())
}

fn parse_initial_state(initial_state: &str) -> Result<TokenizerState, Error> {
    match initial_state {
        "Data state" => Ok(TokenizerState::DataState),
        "PLAINTEXT state" => Ok(TokenizerState::PLAINTEXTState),
        "RCDATA state" => Ok(TokenizerState::RCDATAState),
        "RAWTEXT state" => Ok(TokenizerState::RAWTEXTState),
        "Script data state" => Ok(TokenizerState::ScriptDataState),
        "CDATA section state" => Ok(TokenizerState::CDATASectionState),
        _ => Err(Error::InvalidInitialState),
    }
}

fn serialize_token(
    token: Token,
    tokenizer: &mut Tokenizer,
    serialized_tokens: &mut Vec<String>,
) -> bool {
    match token {
        Token::DOCTYPE(doctype) => {
            let name = doctype
                .name
                .map(|s| format!("\"{s}\""))
                .unwrap_or("null".to_string());
            let public_id = doctype
                .public_ident
                .map(|s| format!("\"{s}\""))
                .unwrap_or("null".to_string());
            let system_id = doctype
                .system_ident
                .map(|s| format!("\"{s}\""))
                .unwrap_or("null".to_string());
            let force_quirks = doctype.force_quirks;

            serialized_tokens.push(format!(
                "[\"DOCTYPE\", {}, {}, {}, {:?}]",
                unicode_escape(&name),
                unicode_escape(&public_id),
                unicode_escape(&system_id),
                !force_quirks,
            ));
        },
        Token::Tag(tagdata) if tagdata.opening => {
            let attributes = tagdata
                .attributes
                .iter()
                .map(|(key, value)| {
                    format!(
                        "\"{}\": \"{}\"",
                        unicode_escape(&key.to_string()),
                        unicode_escape(&value.to_string())
                    )
                })
                .collect::<Vec<String>>()
                .join(",");
            let serialized_token = if tagdata.self_closing {
                format!(
                    "[\"StartTag\", \"{}\", {{{attributes}}}, true]",
                    unicode_escape(&tagdata.name.to_string()),
                )
            } else {
                format!(
                    "[\"StartTag\", \"{}\", {{{attributes}}}]",
                    unicode_escape(&tagdata.name.to_string()),
                )
            };
            serialized_tokens.push(serialized_token);
        },
        Token::Tag(tagdata) if !tagdata.opening => {
            serialized_tokens.push(format!(
                "[\"EndTag\", \"{}\"]",
                unicode_escape(&tagdata.name.to_string()),
            ));
        },
        Token::Comment(comment) => {
            serialized_tokens.push(format!("[\"Comment\", \"{}\"]", unicode_escape(&comment)));
        },
        Token::EOF => {
            return true;
        },
        Token::Character(c) => {
            // Collect all adjacent character tokens
            let mut data = c.to_string();
            loop {
                match tokenizer.next() {
                    Some(Token::Character(c)) => data.push(c),
                    Some(other) => {
                        serialized_tokens
                            .push(format!("[\"Character\", \"{}\"]", unicode_escape(&data)));
                        return serialize_token(other, tokenizer, serialized_tokens);
                    },
                    None => {
                        return true;
                    },
                }
            }
        },
        _ => unreachable!(),
    }
    false
}
