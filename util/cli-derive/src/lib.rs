#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use proc_macro::{Delimiter, Diagnostic, Ident, Level, Span, TokenStream, TokenTree};
use std::iter::Peekable;

// Yo dawg, i heard you like macros
// So i used macros to write your macros
macro_rules! punct_or_error {
    ($token_stream: ident, $punct: expr) => {
        match $token_stream.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == $punct => Ok(punct),
            Some(other) => Err(MacroError::UnexpectedToken(other.span())),
            None => Err(MacroError::UnexpectedEndOfTokens),
        }
    };
}

macro_rules! group_or_error {
    ($token_stream: ident, $delimiter: expr) => {
        match $token_stream.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == $delimiter => Ok(group),
            Some(other) => Err(MacroError::UnexpectedToken(other.span())),
            None => Err(MacroError::UnexpectedEndOfTokens),
        }
    };
}

macro_rules! keyword_or_error {
    ($token_stream: ident, $keyword: expr) => {
        match $token_stream.next() {
            Some(TokenTree::Ident(i)) if i.to_string() == $keyword => Ok(i),
            Some(TokenTree::Ident(i)) => Err(MacroError::UnexpectedKeyword(i.span(), $keyword)),
            Some(other) => Err(MacroError::UnexpectedToken(other.span())),
            None => Err(MacroError::UnexpectedEndOfTokens),
        }
    };
}

macro_rules! ident_or_error {
    ($token_stream: ident) => {
        match $token_stream.next() {
            Some(TokenTree::Ident(i)) => Ok(i),
            Some(other) => Err(MacroError::UnexpectedToken(other.span())),
            None => Err(MacroError::UnexpectedEndOfTokens),
        }
    };
}

macro_rules! next_is_punct {
    ($token_stream: ident, $punct: expr) => {
        match $token_stream.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == $punct => true,
            _ => false,
        }
    };
}

macro_rules! literal_or_error {
    ($token_stream: ident) => {
        match $token_stream.next() {
            Some(TokenTree::Literal(literal)) => Ok(literal),
            Some(other) => Err(MacroError::UnexpectedToken(other.span())),
            None => Err(MacroError::UnexpectedEndOfTokens),
        }
    };
}

#[derive(Debug, Clone)]
struct Argument {
    destination: StructField,
    short_name: String,
    long_name: String,
    _description: String,
    is_optional: bool,
    is_positional: bool,
    is_flag: bool,
}

#[derive(Debug, Clone)]
struct StructField {
    name: String,
}

#[derive(Debug, Default)]
struct HelperAttribute {
    short_name: String,
    long_name: String,
    description: String,
    is_optional: bool,
    is_positional: bool,
    is_flag: bool,
}

#[derive(Clone, Copy, Debug)]
enum MacroError {
    UnexpectedKeyword(Span, &'static str),
    UnexpectedEndOfTokens,
    UnexpectedToken(Span),
    UnknownKey(Span),
}

fn helper_key_requires_value(key: &Ident) -> Result<bool, MacroError> {
    match key.to_string().as_str() {
        "optional" | "positional" | "flag" => Ok(false),
        "long_name" | "short_name" => Ok(true),
        _ => Err(MacroError::UnknownKey(key.span())),
    }
}

fn read_struct_def<I: Iterator<Item = TokenTree>>(tokens: &mut I) -> Result<String, MacroError> {
    keyword_or_error!(tokens, "struct")?;
    ident_or_error!(tokens).map(|ident| ident.to_string())
}

/// Reads a proc-macro helper attribute (`#[argument(optional, long_name=abc)]`)
fn read_helper_attribute<I: Iterator<Item = TokenTree>>(
    tokens: &mut Peekable<I>,
) -> Result<HelperAttribute, MacroError> {
    // First, a leading '#'
    punct_or_error!(tokens, '#')?;

    // Then, a group encapsulated in '[]'
    let mut tokens_in_brackets = group_or_error!(tokens, Delimiter::Bracket)?
        .stream()
        .into_iter()
        .peekable();

    keyword_or_error!(tokens_in_brackets, "argument")?;

    let mut helper_arg_tokens = group_or_error!(tokens_in_brackets, Delimiter::Parenthesis)?
        .stream()
        .into_iter()
        .peekable();

    let mut helper_attribute = HelperAttribute::default();

    // Any number of key-value pairs (a = b)
    while helper_arg_tokens.peek().is_some() {
        let key = ident_or_error!(helper_arg_tokens)?;

        if helper_key_requires_value(&key)? {
            punct_or_error!(helper_arg_tokens, '=')?;
            let value = literal_or_error!(helper_arg_tokens)?;

            match key.to_string().as_str() {
                "description" => helper_attribute.description = value.to_string(), // TODO: We'll want to parse full string literals here
                "long_name" => helper_attribute.long_name = value.to_string(),
                "short_name" => helper_attribute.short_name = value.to_string(),
                _ => unreachable!(), // NOTE: We check for unknown keys inside helper_key_requires_value
            }
        } else {
            match key.to_string().as_str() {
                "optional" => helper_attribute.is_optional = true,
                "positional" => helper_attribute.is_positional = true,
                "flag" => helper_attribute.is_flag = true,
                _ => unreachable!(), // NOTE: We check for unknown keys inside helper_key_requires_value
            }
        }

        // Optionally a comma
        match helper_arg_tokens.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                helper_arg_tokens.next();
            },
            _ => {},
        }
    }
    Ok(helper_attribute)
}

fn read_type<I: Iterator<Item = TokenTree>>(tokens: &mut Peekable<I>) -> Result<(), MacroError> {
    // Types consist of an identifier and (possibly recursive) generics.
    // We read an ident and if there's a '<' after that, we read as long as that
    // the corresponding '>' is found (and discard everything inbetween)
    let _ident = ident_or_error!(tokens)?;

    if next_is_punct!(tokens, '<') {
        tokens.next().ok_or(MacroError::UnexpectedEndOfTokens)?;
        let mut balance = 1;

        while balance != 0 {
            if next_is_punct!(tokens, '<') {
                balance += 1;
            } else if next_is_punct!(tokens, '>') {
                balance -= 1;
            }
            tokens.next().ok_or(MacroError::UnexpectedEndOfTokens)?;
        }
    }
    Ok(())
}

fn read_struct_field<I: Iterator<Item = TokenTree>>(
    tokens: &mut Peekable<I>,
) -> Result<StructField, MacroError> {
    let field_name = ident_or_error!(tokens)?.to_string();

    punct_or_error!(tokens, ':')?;

    read_type(tokens)?;

    // Optionally a comma
    if next_is_punct!(tokens, ',') {
        tokens.next();
    }

    Ok(StructField { name: field_name })
}

fn read_arguments<I: Iterator<Item = TokenTree>>(
    tokens: &mut I,
) -> Result<Vec<Argument>, MacroError> {
    // Outer curly braces
    let mut argument_tokens = group_or_error!(tokens, Delimiter::Brace)?
        .stream()
        .into_iter()
        .peekable();

    let mut arguments = vec![];

    while argument_tokens.peek().is_some() {
        let helper = read_helper_attribute(&mut argument_tokens)?;

        let struct_field = read_struct_field(&mut argument_tokens)?;

        arguments.push(Argument {
            destination: struct_field,
            short_name: helper.short_name,
            long_name: helper.long_name,
            is_optional: helper.is_optional,
            is_positional: helper.is_positional,
            is_flag: helper.is_flag,
            _description: helper.description,
        })
    }
    Ok(arguments)
}

#[proc_macro_derive(CommandLineArgumentParser, attributes(argument))]
pub fn derive_argumentparser_wrapper(input: TokenStream) -> TokenStream {
    // Wraps the derive_argumentparser function for easier errorhandling
    match derive_argumentparser(input) {
        Ok(output) => output,
        Err(error) => {
            match error {
                MacroError::UnexpectedToken(span) => {
                    Diagnostic::spanned(span, Level::Error, "Unexpected token").emit();
                },
                MacroError::UnknownKey(span) => {
                    Diagnostic::spanned(span, Level::Error, "Unknown key").emit();
                },
                MacroError::UnexpectedKeyword(span, expected_keyword) => {
                    Diagnostic::spanned(
                        span,
                        Level::Error,
                        format!("Unexpected keyword, expected {expected_keyword}"),
                    )
                    .emit();
                },
                _ => {},
            }
            TokenStream::new()
        },
    }
}

fn derive_argumentparser(input: TokenStream) -> Result<TokenStream, MacroError> {
    let mut tokens = input.into_iter().peekable();

    let struct_ident = read_struct_def(&mut tokens)?;

    let commandline_arguments = read_arguments(&mut tokens)?;

    let positional_args: Vec<Argument> = commandline_arguments
        .iter()
        .filter(|arg| arg.is_positional)
        .cloned()
        .collect();
    let optional_args: Vec<Argument> = commandline_arguments
        .iter()
        .filter(|arg| !arg.is_positional)
        .cloned()
        .collect();

    let short_optional_args_index_map = format!(
        "|name: char| match name {{ {}, _ => None, }}",
        optional_args
            .iter()
            .enumerate()
            .map(|(index, arg)| format!("{} => Some({index})", arg.short_name))
            .collect::<Vec<String>>()
            .join(",")
    );
    let long_optional_args_index_map = format!(
        "|name: &str| match name {{ {}, _ => None, }}",
        optional_args
            .iter()
            .enumerate()
            .map(|(index, arg)| format!("{} => Some({index})", arg.long_name))
            .collect::<Vec<String>>()
            .join(",")
    );

    let num_args_passed_as_options = optional_args.len();
    let num_args_passed_by_position = positional_args.len();

    let positional_field_initializer = positional_args
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            format!(
                "{}: match &positional_arguments[{index}] {{ 
                        Some(val) => val.parse().map_err(|_| ::cli::CommandLineParseError::InvalidArguments)?,
                        None => if {} {{ Default::default() }} else {{ return Err(::cli::CommandLineParseError::MissingRequiredArgument(\"{0}\")) }},
                }}", arg.destination.name, arg.is_optional
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    let optional_field_initializer = optional_args
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            format!(
                "{}: match &options[{index}] {{ 
                        Some(val_or_none) => match val_or_none {{ 
                            Some(val) => {{ {handle_param_passed_with_value} }},
                            None => {{ {handle_param_passed_as_flag} }}
                        }} ,
                        None => {{ {handle_param_not_passed} }},
                }}",
                arg.destination.name,
                handle_param_passed_with_value = if arg.is_optional {
                    "Some(val.parse().map_err(|_| ::cli::CommandLineParseError::InvalidArguments)?)"
                } else if arg.is_flag {
                    "true"
                } else {
                    "val.parse().map_err(|_| ::cli::CommandLineParseError::InvalidArguments)?"
                },
                handle_param_not_passed = if arg.is_optional | arg.is_flag {
                    "Default::default()".to_string()
                } else {
                    format!(
                        "return Err(::cli::CommandLineParseError::MissingRequiredArgument(\"{}\"))",
                        arg.destination.name
                    )
                },
                handle_param_passed_as_flag = if arg.is_flag {
                    "true".to_string()
                } else {
                    format!(
                        "return Err(::cli::CommandLineParseError::NotAFlag(\"{}\"));",
                        arg.destination.name
                    )
                }
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    let option_is_flag = optional_args
        .iter()
        .map(|arg| if arg.is_flag { "true" } else { "false" })
        .collect::<Vec<&'static str>>()
        .join(",");

    let optional_arguments_found = "None,".repeat(num_args_passed_as_options);
    let positional_arguments_found = "None,".repeat(num_args_passed_by_position);

    let autogenerated_code = format!("
        #[automatically_derived]
        impl ::cli::CommandLineArgumentParser for {struct_ident} {{
            fn parse() -> Result<Self, ::cli::CommandLineParseError> {{
                let mut env_args = ::std::env::args();

                // First, find all the options
                let mut options: [Option<Option<String>>; {num_args_passed_as_options}] = [{optional_arguments_found}];
                let mut positional_arguments: [Option<String>;  {num_args_passed_by_position}] = [{positional_arguments_found}];
                let mut option_is_flag: [bool; {num_args_passed_as_options}] = [{option_is_flag}];

                let short_option_index_map = {short_optional_args_index_map};
                let long_option_index_map = {long_optional_args_index_map};

                let mut num_positional_arguments_found = 0;

                // All arguments up the first flag are positional arguments
                let mut parsing_options = true;
                while let Some(arg) = env_args.next() {{
                    if parsing_options {{
                        // Stop parsing the moment we encounter a single double dash
                        // All the subsequent arguments are positional
                        if arg == \"--\" {{
                            parsing_options = false;
                            continue;
                        }}

                        
                        if let Some(argument) = arg.strip_prefix(\"-\") {{
                            if let Some(argument) = argument.strip_prefix(\"-\") {{
                                // Parse long option
                                let argument_index = match long_option_index_map(argument) {{
                                    Some(index) => index,
                                    None => continue, // unknown flags are ignored
                                }};
                                let value = env_args.next().ok_or(::cli::CommandLineParseError::EmptyOption)?;
                                options[argument_index] = Some(Some(value));
                            }} else {{
                                // Parse short option
                                let first_char = argument.chars().nth(0).ok_or(::cli::CommandLineParseError::EmptyOption)?;
                                let argument_index = match short_option_index_map(first_char) {{
                                    Some(index) => index,
                                    None => continue, // unknown flags are ignored
                                }};
    
                                if option_is_flag[argument_index] {{
                                    // Assume that all the other characters are also flags
                                    for c in argument.chars() {{
                                        match short_option_index_map(c) {{
                                            Some(index) => options[index] = Some(None),
                                            None => {{}}, // ignore
                                        }}
                                    }}
                                }} else {{
                                    // If the first character is an option which requires a value,
                                    // the either the argument is a single character (and the next argument is it's value)
                                    // or the remaining letters are the arguments
                                    if argument.len() == 1 {{
                                        let value = env_args.next().ok_or(::cli::CommandLineParseError::EmptyOption)?;
                                        options[argument_index] = Some(Some(value));
                                    }}
                                }}
                            }}
                            continue;
                        }}
                    }}

                    // Parse positional argument or ignore if we already have
                    // all the arguments we expected
                    if num_positional_arguments_found < {num_args_passed_by_position} {{
                        positional_arguments[num_positional_arguments_found] = Some(arg);
                        num_positional_arguments_found += 1;
                    }}
                }}

                // Construct the actual struct and return it
                Ok(Self {{
                    {positional_field_initializer}
                    {optional_field_initializer}
                }})
            }}

            fn help() -> &'static str {{
                \"TODO\"
            }}
        }}
    ");

    Ok(autogenerated_code
        .parse::<TokenStream>()
        .expect("Macro produced invalid rust code"))
}
