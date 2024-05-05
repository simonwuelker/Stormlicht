use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
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
        #[automatically_derived]
        impl ::serialize::Deserialize for #struct_ident {
            fn deserialize<T: ::serialize::Deserializer>(deserializer: T) -> Result<Self, T::Error> {
                use ::serialize::deserialization::Error;

                #[allow(non_camel_case_types)]
                enum Field {
                    #(#idents,)*
                }

                impl ::serialize::Deserialize for Field {
                    fn deserialize<T: ::serialize::Deserializer>(deserializer: T) -> Result<Self, T::Error> {
                        struct FieldVisitor;

                        impl ::serialize::Visitor for FieldVisitor {
                            type Value = Field;

                            const EXPECTS: &'static str = #expecting;

                            fn visit_string<E>(&self, value: ::std::string::String) -> Result<Self::Value, E>
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

pub(crate) fn deserialize_enum(input: syn::ItemEnum) -> TokenStream {
    let enum_ident = input.ident;

    let mut variant_arms = vec![];
    let mut variant_idents = vec![];

    for variant in input.variants {
        variant_idents.push(variant.ident.clone());
        let ident = &variant.ident;

        match variant.fields {
            syn::Fields::Unit => {
                let code = quote!(
                    (Variant::#ident, variant_data) => {
                        variant_data.unit_variant()?;

                        #enum_ident::#ident
                    }
                );
                variant_arms.push(code);
            },
            syn::Fields::Named(named_fields) => {
                _ = named_fields;
                todo!("implement deserializing enum struct fields");
            },
            syn::Fields::Unnamed(unnamed_fields) => {
                // Generate some field names so we can refer to them in code
                let field_names: Vec<Ident> = (0..unnamed_fields.unnamed.len())
                    .map(|i| Ident::new(&format!("__field__{}", i), Span::call_site()))
                    .collect();

                let code = quote! {
                    (Variant::#ident, variant_data) => {
                        struct TupleVariantVisitor;

                        impl ::serialize::Visitor for TupleVariantVisitor {
                            type Value = #enum_ident;

                            const EXPECTS: &'static str = "A sequence of values";

                            fn visit_sequence<S>(&self, mut value: S) -> Result<Self::Value, S::Error>
                            where
                                S: SequentialAccess,
                            {
                                #(
                                    let #field_names = value.next_element()?.ok_or(S::Error::expected("at least one more value"))?;
                                )*

                                let parsed_value = #enum_ident::#ident(
                                    #(
                                        #field_names,
                                    )*
                                );

                                Ok(parsed_value)
                            }
                        }

                        variant_data.tuple_variant(TupleVariantVisitor)?
                    }
                };
                variant_arms.push(code);
            },
        }
    }

    quote!(
        #[automatically_derived]
        impl ::serialize::Deserialize for #enum_ident {
            fn deserialize<T: ::serialize::Deserializer>(deserializer: T) -> Result<Self, T::Error> {
                use ::serialize::deserialization::{SequentialAccess, EnumVariantAccess, Error};

                enum Variant {
                    #(
                        #variant_idents,
                    )*
                }

                impl ::serialize::Deserialize for Variant {
                    fn deserialize<T: ::serialize::Deserializer>(deserializer: T) -> Result<Self, T::Error> {
                        struct VariantVisitor;

                        impl ::serialize::Visitor for VariantVisitor {
                            type Value = Variant;

                            const EXPECTS: &'static str = "One of the enum variants";

                            fn visit_string<E>(&self, variant_name: String) -> Result<Self::Value, E>
                            where E: ::serialize::deserialization::Error {
                                let variant = match variant_name.as_str() {
                                    #(
                                        stringify!(#variant_idents) => Variant::#variant_idents,
                                    )*
                                    _ => return Err(Error::unknown_variant(variant_name))
                                };

                                Ok(variant)
                            }
                        }

                        deserializer.deserialize_string(VariantVisitor)
                    }
                }


                struct EnumVisitor;

                impl ::serialize::Visitor for EnumVisitor {
                    type Value = #enum_ident;

                    const EXPECTS: &'static str = "A enum";

                    fn visit_enum<E>(&self, enumeration: E) -> Result<Self::Value, E::Error>
                    where E: ::serialize::deserialization::EnumAccess {
                        let parsed_enum = match enumeration.variant()? {
                            #(
                                #variant_arms,
                            )*
                        };

                        Ok(parsed_enum)

                    }
                }

                deserializer.deserialize_enum(EnumVisitor)
            }
        }
    )
    .into()
}
