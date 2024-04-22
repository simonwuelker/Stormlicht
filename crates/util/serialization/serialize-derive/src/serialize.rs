use proc_macro::TokenStream;
use proc_macro2::Ident;
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
        impl ::serialize::Serialize for #struct_ident {
            fn serialize_to<T: ::serialize::Serializer>(&self, serializer: &mut T) -> Result<(), T::Error> {
                use ::serialize::serialization::SerializeStruct;

                serializer.serialize_struct(|struct_serializer| {
                    #(
                        struct_serializer.serialize_field(
                            stringify!(#idents), self.#idents,
                        )?;
                    )*

                    Ok(())
                })?;

                Ok(())
            }
        }
    )
    .into()
}
