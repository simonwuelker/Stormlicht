use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;

#[proc_macro_derive(Deserialize)]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).expect("Could not parse input as item");

    match item {
        syn::Item::Struct(item_struct) => deserialize_struct(item_struct),
        _ => panic!("Cannot impl deserialize for this kind of item"),
    }
}

fn deserialize_struct(input: syn::ItemStruct) -> TokenStream {
    let struct_ident = &input.ident;

    let syn::Fields::Named(fields) = &input.fields else {
        panic!("Struct does not have named fields");
    };

    let idents: Vec<&Ident> = fields
        .named
        .iter()
        .map(|field| field.ident.as_ref().expect("struct field without ident"))
        .collect();

    let expecting = format!("Any of: {:?}", idents);

    quote!(
        impl ::serialize::Deserialize for #struct_ident {
            fn deserialize<T: ::serialize::Deserializer>(deserializer: &mut T) -> Result<Self, T::Error> {
                enum Field {
                    #(#idents,)*
                }

                struct FieldVisitor;

                impl ::serialize::Visitor for FieldVisitor {
                    type Value = Field;

                    const EXPECTS: &'static str = #expecting;

                    fn visit_string<E>(&self, value: String) -> Result<Self::Value, E>
                    where
                        E: ::serialize::deserialization::Error,
                    {
                        let field = match value.as_str() {
                            #(
                                stringify!(#idents) => Field::#idents,
                            )*
                            _ => return Err(E::unknown_field(value)),
                        };
                        Ok(field)
                    }
                }

                struct StructVisitor;

                impl ::serialize::Visitor for StructVisitor {
                    type Value = #struct_ident;

                    const EXPECTS: &'static str = "a map";

                    fn visit_map<M>(&self, value: M) -> Result<Self::Value, M::Error>
                    where M: ::serialize::deserialization::MapAccess {
                        todo!()
                    }
                }

                deserializer.deserialize_map(StructVisitor)
            }
        }
    )
    .into()
}
