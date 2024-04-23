use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;

pub(crate) fn deserialize_struct(input: syn::ItemStruct) -> TokenStream {
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
                use ::serialize::deserialization::Error;

                #[allow(non_camel_case_types)]
                enum Field {
                    #(#idents,)*
                }

                impl ::serialize::Deserialize for Field {
                    fn deserialize<T: ::serialize::Deserializer>(deserializer: &mut T) -> Result<Self, T::Error> {
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

                        deserializer.deserialize_string(FieldVisitor)
                    }
                }

                struct StructVisitor;

                impl ::serialize::Visitor for StructVisitor {
                    type Value = #struct_ident;

                    const EXPECTS: &'static str = "a map";

                    fn visit_map<M>(&self, mut value: M) -> Result<Self::Value, M::Error>
                    where M: ::serialize::deserialization::MapAccess {
                        #(
                            let mut #idents = None;
                        )*


                        loop {
                            let Some(key) = value.next_key()? else {
                                break;
                            };

                            match key {
                                #(
                                    Field::#idents => #idents = Some(value.next_value()?),
                                )*
                            }
                        }

                        let instance = Self::Value {
                            #(
                                #idents: #idents.ok_or(Error::missing_field(stringify!(#idents)))?,
                            )*
                        };

                        Ok(instance)
                    }
                }

                deserializer.deserialize_struct(StructVisitor)
            }
        }
    )
    .into()
}
