use quote::quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldConstraints {
    Ordered,
    Unordered,
    OptionalUnordered,
}
// There is currently a bug in rustfmt that causes formatting to fail in this function
// https://github.com/rust-lang/rustfmt/issues/5744
#[rustfmt::skip] 
#[proc_macro_derive(
    CSSProperty,
    attributes(keyword, ordered, unordered, optional_unordered)
)]
pub fn cssproperty_derive(input: TokenStream) -> TokenStream {
    let enum_def: syn::ItemEnum = syn::parse(input).expect("Not a valid rust enum");
    let enum_name = enum_def.ident;

    let mut keywords = vec![];
    let mut value_types = vec![];
    let mut struct_types = vec![];

    // Iterate over all the enum variants
    'variants: for variant in &enum_def.variants {
        match &variant.fields {
            syn::Fields::Named(named_fields) => {
                // Find out if one of the three annotations exists on the field:
                // * "ordered" (All fields required, in exact order)
                // * "unordered" (All fields required, any order) (https://drafts.csswg.org/css-values-4/#comb-all)
                // * "optional_unordered" (At least one field required, any order)
                for attribute in &variant.attrs {
                    if let syn::Meta::Path(path) = &attribute.meta {
                        let path_str = path.get_ident().map(|i| i.to_string());
                        let constraints = match path_str.as_deref() {
                            Some("ordered") => FieldConstraints::Ordered,
                            Some("unordered") => FieldConstraints::Unordered,
                            Some("optional_unordered") => FieldConstraints::OptionalUnordered,
                            _ => continue,
                        };

                        struct_types.push((constraints, named_fields, &variant.ident));
                        break;
                    }
                }
            },
            syn::Fields::Unnamed(unnamed_fields) => {
                if let Some(value_type) = unnamed_fields.unnamed.first() {
                    if let syn::Type::Path(type_path) = &value_type.ty {
                        let value_type_name = type_path.path.get_ident().expect(
                            "CSS Property value type must have simple identifier (no '::')",
                        );
                        value_types.push((value_type_name, &variant.ident));

                        continue 'variants;
                    } else {
                        panic!("CSS Properties must be simple types");
                    }
                } else {
                    panic!("CSS Properties can only contain one single value");
                }
            },
            syn::Fields::Unit => {
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
                                keywords.push((keyword, &variant.ident));

                                continue 'variants; // We're done parsing this variant
                            } else {
                                panic!("Invalid use of keyword attribute: Only string literals are accepted as values");
                            }
                        }
                    }
                }
                panic!("Unit enum variants must have a #[keyword] attribute");
            },
        }
    }

    // These are the match arms in a match block that map from a keyword (&str)
    // to the corresponding value type. For example:
    // ```
    // "foo" => Self::Foo
    // ```
    let keyword_match = keywords
        .iter()
        .map(|(keyword, variant)| quote!(static_interned!(#keyword) => Self::#variant,))
        .collect::<TokenStream2>();

    // For each unit struct variant (For example, "::Foo(ContainedType)" ),
    // we try and parse the contained type and if it succeeds, return that variant
    // with the parsed value
    let parse_unit_variants = value_types
        .iter()
        .map(|(type_name, variant)| {
            quote! {
                if let Some(value) = parser.parse_optional_value(#type_name::parse) {
                    return Ok(Self::#variant(value));
                }
            }
        })
        .collect::<TokenStream2>();

    let parse_struct_variants = struct_types
        .iter()
        .map(|(constraint, named_fields, variant)| {
            let parse_struct = match constraint {
                FieldConstraints::Ordered => {
                    let field_parsing = named_fields
                        .named
                        .iter()
                        .map(|field| {
                            let field_name = field
                                .ident
                                .as_ref()
                                .expect("Named fields always have a name");
                            let field_type = &field.ty;

                            quote!(#field_name: <#field_type as CSSParse>::parse(parser)?,)
                        })
                        .collect::<TokenStream2>();
                    quote!(
                        Ok(Self::#variant {
                            #field_parsing
                        })
                    )
                },
                FieldConstraints::Unordered | FieldConstraints::OptionalUnordered => {
                    let number_of_fields = named_fields.named.len();

                    let field_idents = (0..number_of_fields)
                        .map(|index| {
                            syn::Ident::new(&format!("field_{index}"), proc_macro2::Span::call_site())
                        })
                        .collect::<Vec<syn::Ident>>();

                    // Initially, every field is None (aka has not yet been parsed)
                    let field_initialization = field_idents
                        .iter()
                        .map(|ident| {
                            quote!(
                                let mut #ident = None;
                            )
                        })
                        .collect::<TokenStream2>();

                    // Code to try and parse each field of the struct once, aborting early
                    // if parsing of a field succeeds. If parsing succeeds, the value is written
                    // into the parsed_fields array
                    let parse_fields = field_idents
                        .iter()
                        .zip(&named_fields.named)
                        .map(|(field_ident, named_field)| {
                            let field_type = &named_field.ty;
                            if constraint == &FieldConstraints::OptionalUnordered {
                                quote!(
                                    if #field_ident.is_none() {
                                        if let Some(parsed_value) = <#field_type as CSSParse>::parse(parser)? {
                                            #field_ident = Some(parsed_value);
                                            continue;
                                        }
                                    }
                                )
                            } else {
                                quote!(
                                    if #field_ident.is_none() {
                                        if let Some(parsed_value) = parser.parse_optional_value(<#field_type as CSSParse>::parse) {
                                            #field_ident = Some(parsed_value);
                                            continue;
                                        }
                                    }
                                )
                            }
                        })
                        .collect::<TokenStream2>();

                    let field_readout = field_idents
                        .iter()
                        .zip(&named_fields.named)
                        .map(|(field_ident, named_field)| {
                            let field_name = named_field.ident.as_ref().unwrap();
                            if constraint == &FieldConstraints::OptionalUnordered {
                                quote!(
                                    #field_name: #field_ident,
                                )
                            } else {
                                quote!(
                                    #field_name: #field_ident.unwrap(),
                                )
                            }
                        })
                        .collect::<TokenStream2>();

                    let handle_nothing_parsed_case = if constraint == &FieldConstraints::OptionalUnordered {
                        // If our fields are optional, then we are simply done parsing
                        quote!(
                            break;
                        )
                    } else {
                        // If our fields are not optional, then this is an error and we 
                        quote!(
                            // If we did not continue yet, then none of the fields parsed successfully
                            // which is an error
                            return Err(ParseError);
                        )
                    };

                    let verify_parsed_value = if constraint == &FieldConstraints::OptionalUnordered {
                        let all_fields_are_none = field_idents
                        .iter()
                        .map(|ident| {
                            quote!(
                                #ident.is_none()
                            )
                        })
                        .reduce(|acc, e| quote!(#acc && #e))
                        .unwrap_or_default();
                        quote!(
                            if #all_fields_are_none {
                                return Err(ParseError);
                            }
                        )
                    } else {
                        TokenStream2::new()
                    };

                    quote!(
                        #field_initialization

                        for _ in 0..#number_of_fields {
                            #parse_fields
                            #handle_nothing_parsed_case
                        }

                        #verify_parsed_value

                        Ok(Self::#variant {
                            #field_readout
                        })
                    )
                },
            };

            quote!(
                if let Some(parsed_value) = parser.parse_optional_value(|parser| {#parse_struct}) {
                    return Ok(parsed_value);
                }
            )
        })
        .collect::<TokenStream2>();

    let parse_keywords = if keywords.is_empty() {
        TokenStream2::new()
    } else {
        quote!(
            use string_interner::{static_interned, static_str};
            let parsed_keyword = parser.parse_optional_value(|parser| {
                if let Some(Token::Ident(identifier)) = parser.next_token() {
                    let parsed_keyword = match identifier {
                        #keyword_match
                        _ => return Err(ParseError),
                    };
                    parser.skip_whitespace();
                    Ok(parsed_keyword)
                } else {
                    Err(ParseError)
                }
            });

            if let Some(variant) = parsed_keyword {
                return Ok(variant);
            }
        )
    };

    // Put everything together
    quote!(
        #[automatically_derived]
        impl<'a> CSSParse<'a> for #enum_name {
            fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
                #parse_keywords

                #parse_unit_variants

                #parse_struct_variants

                Err(ParseError)
            }
        }
    )
    .into()
}
