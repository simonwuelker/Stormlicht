#![feature(proc_macro_diagnostic)]

use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Lit, Meta, NestedMeta};

#[derive(Debug, Clone)]
struct Argument {
    destination: String,
    short_name: String,
    long_name: String,
    description: String,
    /// Whether the argument can be omitted.
    /// If omitted, `Default::default()` is used.
    may_be_omitted: bool,
    /// Whether the argument is parsed based on its position in the argument stream.
    is_positional: bool,
    /// Whether the argument is a flag (an argument with a boolean value)
    /// In this case, the value may be omitted, in which case it is implicitly true.
    is_flag: bool,
}

#[proc_macro_derive(CommandLineArgumentParser, attributes(argument))]
pub fn derive_argumentparser_wrapper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input struct
    let input = parse_macro_input!(input as DeriveInput);
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => {
            panic!("CommandLineArgumentParser can only be derived for structs with named fields",);
        },
    };

    // Map the struct fields to our internal argument struct
    let mut commandline_arguments = vec![];
    for field in fields {
        for attr in field.attrs {
            if let Ok(Meta::List(list)) = attr.parse_meta() {
                if list.path.segments.len() == 1 && list.path.segments[0].ident == "argument" {
                    let mut short_name = None;
                    let mut long_name = None;
                    let mut description = None;
                    let mut may_be_omitted = false;
                    let mut is_positional = false;
                    let mut is_flag = false;

                    for value in list.nested {
                        match value {
                            NestedMeta::Meta(Meta::Path(keyword)) => {
                                match keyword
                                    .get_ident()
                                    .expect("Expected ident path")
                                    .to_string()
                                    .as_str()
                                {
                                    "may_be_omitted" => may_be_omitted = true,
                                    "positional" => is_positional = true,
                                    "flag" => is_flag = true,
                                    other => panic!("Unknown key: {other}"),
                                }
                            },
                            NestedMeta::Meta(Meta::NameValue(name_value)) => {
                                let key = name_value
                                    .path
                                    .get_ident()
                                    .expect("Expected a single ident as argument key")
                                    .to_string();
                                let value = match name_value.lit {
                                    Lit::Str(string_literal) => string_literal.value(),
                                    Lit::Char(char_literal) => char_literal.value().to_string(),
                                    _ => panic!("Expected string as value"),
                                };
                                match key.as_str() {
                                    "short_name" => short_name = Some(value),
                                    "long_name" => long_name = Some(value),
                                    "description" => description = Some(value),
                                    other => panic!("Unknown key: {other}"),
                                }
                            },
                            _ => panic!("Invalid argument descriptor"),
                        }
                    }

                    if is_flag && may_be_omitted {
                        panic!("Redundant parameter: Flags may always be omitted.");
                    } else if is_flag && is_positional {
                        panic!("Flags cannot be positional");
                    }

                    let argument = Argument {
                        destination: field.ident.expect("Expected field identifier").to_string(),
                        short_name: short_name.expect("Missing argument short name"),
                        long_name: long_name.expect("Missing argument long name"),
                        description: description.expect("Missing argument description"),
                        may_be_omitted,
                        is_positional,
                        is_flag,
                    };
                    commandline_arguments.push(argument);
                    break;
                }
            }
        }
    }

    let short_optional_args_index_map = format!(
        "|name: char| match name {{ {}, _ => None, }}",
        commandline_arguments
            .iter()
            .enumerate()
            .map(|(index, arg)| format!("\'{}\' => Some({index})", arg.short_name))
            .collect::<Vec<String>>()
            .join(",")
    );
    let long_optional_args_index_map = format!(
        "|name: &str| match name {{ {}, _ => None, }}",
        commandline_arguments
            .iter()
            .enumerate()
            .map(|(index, arg)| format!("\"{}\" => Some({index})", arg.long_name))
            .collect::<Vec<String>>()
            .join(",")
    );

    let num_arguments = commandline_arguments.len();

    let field_initializer = commandline_arguments
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            format!(
                "{}: match &arguments[{index}] {{ 
                        Some(val_or_none) => match val_or_none {{ 
                            Some(val) => {{ {handle_param_passed_with_value} }},
                            None => {{ {handle_param_passed_as_flag} }}
                        }} ,
                        None => {{ {handle_param_not_passed} }},
                }}",
                arg.destination,
                handle_param_passed_with_value = if arg.is_flag {
                    "true"
                } else if arg.may_be_omitted {
                    "Some(val.parse().map_err(|_| ::cli::CommandLineParseError::InvalidArguments)?)"
                } else {
                    "val.parse().map_err(|_| ::cli::CommandLineParseError::InvalidArguments)?"
                },
                handle_param_not_passed = if arg.may_be_omitted | arg.is_flag {
                    "Default::default()".to_string()
                } else {
                    format!(
                        "return Err(::cli::CommandLineParseError::MissingRequiredArgument(\"{}\"))",
                        arg.destination
                    )
                },
                handle_param_passed_as_flag = if arg.is_flag {
                    "true".to_string()
                } else {
                    format!(
                        "return Err(::cli::CommandLineParseError::NotAFlag(\"{}\"));",
                        arg.destination
                    )
                }
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    let argument_is_flag = commandline_arguments
        .iter()
        .map(|arg| if arg.is_flag { "true" } else { "false" })
        .collect::<Vec<&'static str>>()
        .join(",");

    let arguments_found = "None,".repeat(num_arguments);
    let num_positional_arguments = commandline_arguments
        .iter()
        .filter(|arg| arg.is_positional)
        .count();
    let argument_indices = commandline_arguments
        .iter()
        .enumerate()
        .filter(|(_, argument)| argument.is_positional)
        .map(|(index, _)| index.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let help_msg = build_help_msg(&commandline_arguments);
    let struct_ident = input.ident.to_string();
    let autogenerated_code = format!("
    #[automatically_derived]
    impl ::cli::CommandLineArgumentParser for {struct_ident} {{
            fn parse() -> Result<Self, ::cli::CommandLineParseError> {{
                let mut env_args = ::std::env::args().skip(1);

                // First, find all the options
                // None => the argument was not passed
                // Some(None) => the argument was passed without a value
                // Some(value) => the argument passed with value
                let mut arguments: [Option<Option<String>>; {num_arguments}] = [{arguments_found}];
                
                // Map from the index of a positional argument to it's respective index in the argument array
                let mut positional_argument_map: [usize; {num_positional_arguments}] = [{argument_indices}];
                
                let mut argument_is_flag: [bool; {num_arguments}] = [{argument_is_flag}];

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
                                let (argument_name, argument_value) = if argument.contains('=') {{
                                    let (name, value) = argument.split_once('=').unwrap();
                                    (name, Some(value))
                                }} else {{
                                    (argument, None)
                                }}
                                ;
                                // Parse long option
                                let argument_index = match long_option_index_map(argument_name) {{
                                    Some(index) => index,
                                    None => continue, // unknown flags are ignored
                                }};

                                if argument_is_flag[argument_index] {{
                                    arguments[argument_index] = Some(None);
                                }} else {{
                                    // If no value was provided with '--key=value' syntax,
                                    // we read the next argument and use that instead
                                    let value = if let Some(value) = argument_value {{
                                        value.to_string()
                                    }} else {{
                                        env_args.next().ok_or(::cli::CommandLineParseError::EmptyOption)?
                                    }};

                                    arguments[argument_index] = Some(Some(value));
                                }}
                            }} else {{
                                // Parse short option
                                let first_char = argument.chars().nth(0).ok_or(::cli::CommandLineParseError::EmptyOption)?;
                                let argument_index = match short_option_index_map(first_char) {{
                                    Some(index) => index,
                                    None => continue, // unknown flags are ignored
                                }};
    
                                if argument_is_flag[argument_index] {{
                                    // Assume that all the other characters are also flags
                                    for c in argument.chars() {{
                                        match short_option_index_map(c) {{
                                            Some(index) => arguments[index] = Some(None),
                                            None => {{}}, // ignore
                                        }}
                                    }}
                                }} else {{
                                    // If the first character is an option which requires a value,
                                    // the either the argument is a single character (and the next argument is it's value)
                                    // or the remaining letters are the arguments
                                    if argument.len() == 1 {{
                                        let value = env_args.next().ok_or(::cli::CommandLineParseError::EmptyOption)?;
                                        arguments[argument_index] = Some(Some(value));
                                    }}
                                }}
                            }}
                            continue;
                        }}
                    }}

                    // Parse positional argument or ignore if we already have
                    // all the arguments we expected
                    if num_positional_arguments_found < positional_argument_map.len() {{
                        arguments[positional_argument_map[num_positional_arguments_found]] = Some(Some(arg));
                        num_positional_arguments_found += 1;
                    }}
                }}

                // Construct the actual struct and return it
                Ok(Self {{
                    {field_initializer}
                }})
            }}

            fn help() -> &'static str {{
                concat!(\"Usage: \", env!(\"CARGO_BIN_NAME\"), \"{help_msg}\")
            }}
        }}
    ");

    autogenerated_code
        .parse::<proc_macro::TokenStream>()
        .expect("Macro produced invalid rust code")
}

fn build_help_msg(arguments: &[Argument]) -> String {
    let options_names = arguments
        .iter()
        .map(|arg| format!("   -{} or --{}", arg.short_name, arg.long_name))
        .collect::<Vec<String>>();
    let longest_name = options_names
        .iter()
        .map(String::len)
        .max()
        .unwrap_or_default();
    let description_indentation = longest_name + 5;

    let options_descriptions = arguments
        .iter()
        .zip(options_names)
        .map(|(arg, name)| {
            format!(
                "{}{}{}",
                name,
                " ".repeat(description_indentation - name.len()),
                arg.description
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let positional_arg_names = arguments
        .iter()
        .filter(|arg| arg.is_positional)
        .map(|arg| {
            if arg.may_be_omitted {
                format!("[{}]", arg.long_name)
            } else {
                arg.long_name.clone()
            }
        })
        .collect::<Vec<String>>()
        .join(" ");
    format!(
        "[ options ] {positional_arg_names}\n\twhere options include:\n\n{options_descriptions}"
    )
}
