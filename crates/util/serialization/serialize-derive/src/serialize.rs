use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) fn serialize_struct(input: syn::ItemStruct) -> TokenStream {
    let struct_ident = &input.ident;

    let syn::Fields::Named(fields) = &input.fields else {
        panic!("Struct does not have named fields");
    };

    let idents: Vec<&Ident> = fields
        .named
        .iter()
        .map(|field| field.ident.as_ref().expect("struct field without ident"))
        .collect();

    quote!(
        #[automatically_derived]
        impl ::serialize::Serialize for #struct_ident {
            fn serialize_to<T: ::serialize::Serializer>(&self, serializer: &mut T) -> Result<(), T::Error> {
                use ::serialize::serialization::SerializeStruct;

                let mut struct_serializer = serializer.serialize_struct()?;
                #(
                    struct_serializer.serialize_field(
                        stringify!(#idents), &self.#idents,
                    )?;
                )*
                struct_serializer.finish()?;

                Ok(())
            }
        }
    )
    .into()
}

pub(crate) fn serialize_enum(input: syn::ItemEnum) -> TokenStream {
    let enum_ident = &input.ident;

    let mut serialize_arms = vec![];
    for variant in input.variants {
        let ident = variant.ident;

        match variant.fields {
            syn::Fields::Unit => {
                let code = quote!(
                    Self::#ident => serializer.serialize_enum(stringify!(#ident))?
                );
                serialize_arms.push(code);
            },
            syn::Fields::Named(named_fields) => {
                let field_names: Vec<&Ident> =
                    named_fields.named.iter().flat_map(|f| &f.ident).collect();
                let code = quote! {
                    Self::#ident{ #(#field_names,)* } => {
                        let mut struct_serializer = serializer.serialize_struct_enum(stringify!(#ident))?;
                        #(
                            struct_serializer.serialize_field(stringify!(#field_names), #field_names)?;
                        )*
                        struct_serializer.finish()?;
                    }
                };
                serialize_arms.push(code);
            },
            syn::Fields::Unnamed(unnamed_fields) => {
                // Generate some field names so we can refer to them in code
                let field_names: Vec<Ident> = (0..unnamed_fields.unnamed.len())
                    .map(|i| Ident::new(&format!("__field__{}", i), Span::call_site()))
                    .collect();

                let code = quote! {
                    Self::#ident(#(#field_names,)*) => {
                        let mut tuple_serializer = serializer.serialize_tuple_enum(stringify!(#ident))?;
                        #(
                            tuple_serializer.serialize_element(#field_names)?;
                        )*
                        tuple_serializer.finish()?;
                    }
                };
                serialize_arms.push(code);
            },
        }
    }

    quote!(
        #[automatically_derived]
        impl ::serialize::Serialize for #enum_ident {
            fn serialize_to<T: ::serialize::Serializer>(&self, serializer: &mut T) -> Result<(), T::Error> {
                use ::serialize::serialization::SerializeTupleVariant;
                use ::serialize::serialization::SerializeStructVariant;

                match self {
                    #(
                        #serialize_arms,
                    )*
                }

                Ok(())
            }
        }
    )
    .into()
}
