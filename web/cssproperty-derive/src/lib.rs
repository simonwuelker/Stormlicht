use std::str::FromStr;

use proc_macro::TokenStream;

#[derive(Clone, Debug)]
struct Keyword {
    name: String,
    variant: String,
}

#[derive(Clone, Debug)]
struct ValueType {
    type_name: String,
    variant: String,
}

#[proc_macro_derive(CSSProperty, attributes(keyword))]
pub fn cssproperty_derive(input: TokenStream) -> TokenStream {
    let enum_def: syn::ItemEnum = syn::parse(input).expect("Not a valid rust enum");
    let enum_name = enum_def.ident.to_string();

    let mut keywords: Vec<Keyword> = vec![];
    let mut value_types: Vec<ValueType> = vec![];

    // Iterate over all the enum variants
    'variants: for variant in &enum_def.variants {
        // Check if there is a "keyword" variant
        for attribute in &variant.attrs {
            if let syn::Meta::NameValue(name_value) = &attribute.meta {
                if matches!(
                    name_value
                        .path
                        .get_ident()
                        .map(|i| i.to_string())
                        .as_deref(),
                    Some("keyword")
                ) {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(keyword),
                        ..
                    }) = &name_value.value
                    {
                        keywords.push(Keyword {
                            name: keyword.value(),
                            variant: variant.ident.to_string(),
                        });
                        continue 'variants; // We're done parsing this variant
                    } else {
                        panic!("Invalid use of keyword attribute: Only string literals are accepted as values");
                    }
                }
            }
        }

        // Check if its an enum variant, like ::Color(Color)
        if let syn::Fields::Unnamed(unnamed_fields) = &variant.fields {
            if let Some(value_type) = unnamed_fields.unnamed.first() {
                if let syn::Type::Path(type_path) = &value_type.ty {
                    let value_type_name = type_path
                        .path
                        .get_ident()
                        .expect("CSS Property value type must have simple identifier (no '::')");
                    value_types.push(ValueType {
                        type_name: value_type_name.to_string(),
                        variant: variant.ident.to_string(),
                    });
                    continue 'variants;
                } else {
                    panic!("CSS Properties must be simple types");
                }
            } else {
                panic!("CSS Properties can only contain one single value");
            }
        }

        panic!("CSS Property variant must either have an associated parseable value or a #[keyword] annotation");
    }

    let keyword_match = keywords
        .iter()
        .map(|keyword| format!("\"{}\" => Ok(Self::{})", keyword.name, keyword.variant))
        .collect::<Vec<String>>()
        .join(",");

    let value_type_code = value_types
        .iter()
        .map(|value_type| {
            format!(
                "if let Some(value) = parser.parse_optional_value({}::parse) {{
                    return Ok(Self::{}(value));
                }}",
                value_type.type_name, value_type.variant,
            )
        })
        .collect::<String>();

    let css_property_impl = format!(
        "
    impl<'a> CSSParse<'a> for {enum_name} {{
        fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {{
            let parsed_keyword = parser.parse_optional_value(|parser| {{
                if let Some(Token::Ident(identifier)) = parser.next_token() {{
                    match identifier.as_ref() {{
                        {keyword_match},
                        _ => Err(ParseError),
                    }}
                }} else {{
                    Err(ParseError)
                }}
            }});

            if let Some(variant) = parsed_keyword {{
                return Ok(variant);
            }}

            {value_type_code}

            Err(ParseError)
        }}
    }}"
    );

    TokenStream::from_str(&css_property_impl).expect("Proc macro produced invalid rust code")
}
