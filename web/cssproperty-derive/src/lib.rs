use quote::quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro_derive(CSSProperty, attributes(keyword))]
pub fn cssproperty_derive(input: TokenStream) -> TokenStream {
    let enum_def: syn::ItemEnum = syn::parse(input).expect("Not a valid rust enum");
    let enum_name = enum_def.ident;

    let mut keywords = vec![];
    let mut value_types = vec![];

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
                        keywords.push((keyword, &variant.ident));

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
                    value_types.push((value_type_name, &variant.ident));

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
        .map(|(keyword, variant)| quote!(#keyword => Ok(Self::#variant),))
        .collect::<TokenStream2>();

    let value_type_code = value_types
        .iter()
        .map(|(type_name, variant)| {
            quote! {
                if let Some(value) = parser.parse_optional_value(#type_name::parse) {
                    return Ok(Self::#variant(value));
                }
            }
        })
        .collect::<TokenStream2>();

    quote!(
        impl<'a> CSSParse<'a> for #enum_name {
            fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
                let parsed_keyword = parser.parse_optional_value(|parser| {
                    if let Some(Token::Ident(identifier)) = parser.next_token() {
                        match identifier.as_ref() {
                            #keyword_match
                            _ => Err(ParseError),
                        }
                    } else {
                        Err(ParseError)
                    }
                });

                if let Some(variant) = parsed_keyword {
                    return Ok(variant);
                }

                #value_type_code

                Err(ParseError)
            }
        }
    )
    .into()
}
