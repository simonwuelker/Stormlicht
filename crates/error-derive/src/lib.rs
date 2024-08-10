use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Error, attributes(msg))]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let item: syn::ItemEnum = syn::parse(input).expect("Could not parse input as enum");

    let name = item.ident;
    let mut variant_displays = vec![];
    let mut from_impls = vec![];
    let mut variant_sources = vec![];

    for variant in &item.variants {
        let ident = &variant.ident;

        let display_attribute = variant
            .attrs
            .iter()
            .flat_map(|attr| match &attr.meta {
                syn::Meta::NameValue(name_value) => Some(name_value),
                _ => None,
            })
            .flat_map(|attr| {
                let ident = attr.path.get_ident()?.to_string();

                Some((ident, &attr.value))
            })
            .find(|(name, _)| name == "msg")
            .map(|(_, value)| value);

        let Some(display_value) = display_attribute else {
            panic!("need display attribute");
        };

        match &variant.fields {
            syn::Fields::Unit => {
                variant_displays.push(quote!(Self::#ident => (#display_value).fmt(f)));
            },
            syn::Fields::Unnamed(unnamed_fields) => {
                if unnamed_fields.unnamed.len() != 1 {
                    panic!("Need exactly one field");
                }

                let field = &unnamed_fields.unnamed[0];
                let ty = &field.ty;

                from_impls.push(quote!(
                    #[automatically_derived]
                    impl From<#ty> for #name {
                        fn from(value: #ty) -> Self {
                            Self::#ident(value)
                        }
                    }
                ));
                variant_displays.push(quote!(Self::#ident(_) => (#display_value).fmt(f)));
                variant_sources.push(quote!(Self::#ident(ref value) => Some(value)));
            },
            syn::Fields::Named(_) => panic!("named fields are not allowed"),
        }
    }

    quote!(
        #[automatically_derived]
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                match self {
                    #(
                        #variant_displays,
                    )*
                }
            }
        }

        #(
            #from_impls
        )*

        #[automatically_derived]
        impl ::std::error::Error for #name {
            fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
                match self {
                    #(
                        #variant_sources,
                    )*
                    _ => None,
                }
            }
        }
    )
    .into()
}
