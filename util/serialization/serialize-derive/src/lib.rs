use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;

#[proc_macro_derive(Deserialize)]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let input_struct: syn::ItemStruct =
        syn::parse(input).expect("Proc macro can only be applied to structs");
    let struct_ident = &input_struct.ident;

    if let syn::Fields::Named(fields) = &input_struct.fields {
        let idents: Vec<&Ident> = fields
            .named
            .iter()
            .map(|field| field.ident.as_ref().expect("struct field without ident"))
            .collect();

        quote!(
            impl ::serialize::Deserialize for #struct_ident {
                fn deserialize<T: ::serialize::Deserializer>(deserializer: &mut T) -> Result<Self, T::Error> {
                    deserializer.start_struct()?;

                    let deserialized_instance = Self {
                        #(
                            #idents: deserializer.deserialize_field(stringify!(#idents))?,
                        )*
                    };
                    deserializer.end_struct()?;
                    return Ok(deserialized_instance);
                }
            }
        )
        .into()
    } else {
        panic!("Struct does not have named fields");
    }
}
