use proc_macro::TokenStream;

mod deserialize;
mod serialize;

#[proc_macro_derive(Deserialize)]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).expect("Could not parse input as item");

    match item {
        syn::Item::Struct(item_struct) => deserialize::deserialize_struct(item_struct),
        syn::Item::Enum(item_enum) => deserialize::deserialize_enum(item_enum),
        _ => panic!("Cannot impl Deserialize for this kind of item"),
    }
}

#[proc_macro_derive(Serialize)]
pub fn serialize(input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).expect("Could not parse input as item");

    match item {
        syn::Item::Struct(item_struct) => serialize::serialize_struct(item_struct),
        syn::Item::Enum(item_enum) => serialize::serialize_enum(item_enum),
        _ => panic!("Cannot impl Serialize for this kind of item"),
    }
}
