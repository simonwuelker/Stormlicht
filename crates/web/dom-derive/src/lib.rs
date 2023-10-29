use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn inherit(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the child struct declaration
    let mut struct_declaration: syn::ItemStruct = syn::parse(item).unwrap();

    if attr.is_empty() {
        // This is a root object that does not inherit from anything
        return quote!(
            #[repr(C)]
            #[derive(Default)]
            #struct_declaration
        )
        .into();
    }

    // The attribute must only contain the name of the struct to inherit
    let parent_type_ident: syn::Ident =
        syn::parse(attr).expect("Expected a struct ident as inherit argument");

    // Inject our parent struct field
    // TODO: the way we parse the struct field is a bit janky,
    // not sure if syn supports a better way. How does
    // one call Field::parse_named()?
    let injected_named_fields: syn::FieldsNamed = syn::parse_quote!(
        {__parent: #parent_type_ident}
    );
    let injected_field = injected_named_fields.named.into_iter().nth(0).unwrap();

    match &mut struct_declaration.fields {
        syn::Fields::Named(ref mut named_fields) => named_fields.named.insert(0, injected_field),
        syn::Fields::Unnamed(ref mut unnamed_fields) => {
            unnamed_fields.unnamed.insert(0, injected_field)
        },
        syn::Fields::Unit => panic!("Unit structs cannot inherit"),
    }

    let struct_ident = &struct_declaration.ident;
    quote!(
        #[repr(C)]
        #[derive(Default)]
        #struct_declaration

        #[automatically_derived]
        impl ::std::ops::Deref for #struct_ident {
            type Target = #parent_type_ident;

            fn deref(&self) -> &Self::Target {
                &self.__parent
            }
        }

        #[automatically_derived]
        impl ::std::ops::DerefMut for #struct_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.__parent
            }
        }
    )
    .into()
}
