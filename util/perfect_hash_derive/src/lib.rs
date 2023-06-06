use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, LitStr, Token,
};

struct InputData {
    _const_token: Token![const],
    ident: Ident,
    _eq: Token![=],
    _brackets: token::Bracket,
    items: Punctuated<LitStr, Token![,]>,
    _semicolon: Token![;],
}

impl Parse for InputData {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(InputData {
            _const_token: input.parse()?,
            ident: input.parse()?,
            _eq: input.parse()?,
            _brackets: bracketed!(content in input),
            items: content.parse_terminated(<LitStr as Parse>::parse, Token![,])?,
            _semicolon: input.parse()?,
        })
    }
}

// Ad-Hoc string hashing since string hashing requires some unstable stuff
// otherwise (https://github.com/rust-lang/rust/issues/96762)
// I doubt that its seriously slower, if not faster
const fn str_hash(string: &str) -> u32 {
    let b: u32 = 378551;
    let mut a: u32 = 63689;
    let mut hash: u32 = 0;
    let bytes = string.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        hash = hash.wrapping_mul(a).wrapping_add(bytes[i] as u32);
        a = a.wrapping_mul(b);
        i += 1;
    }

    hash
}

const fn int_hash(mut x: u32) -> u32 {
    x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3b);
    x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3b);
    x = (x >> 16) ^ x;
    x
}

#[proc_macro]
pub fn perfect_set(input: TokenStream) -> TokenStream {
    let input_data: InputData = syn::parse(input).expect("Invalid usage of proc macro");

    let strings: Vec<String> = input_data.items.iter().map(|item| item.value()).collect();
    let size = strings.len();
    // Map all strings into buckets like in a regular hashmap
    let mut first_level_buckets = vec![vec![]; size];
    for string in &strings {
        let hash = str_hash(string);
        first_level_buckets[hash as usize % size].push(hash);
    }

    // Create a way to iterate over the buckets from largest to smallest
    let mut bucket_order: Vec<usize> = (0..size).collect();
    bucket_order.sort_unstable_by_key(|&index| first_level_buckets[index].len());

    // Create a set of secondary hash functions, one for each bucket, that
    let mut occupied_indices = vec![false; size];
    let mut secondary_hash_functions = vec![0; size];

    for bucket_index in bucket_order {
        for secondary_hash_fn in 0.. {
            let mut tmp = occupied_indices.clone();
            if hash_fn_does_not_have_collisions(
                secondary_hash_fn,
                first_level_buckets[bucket_index].as_slice(),
                &mut tmp,
            ) {
                secondary_hash_functions[bucket_index] = secondary_hash_fn;

                // Lock in the occupied indices
                occupied_indices = tmp;

                // continue for the next bucket
                break;
            }
        }
    }

    let second_hash_functions_tokens: TokenStream2 = secondary_hash_functions
        .iter()
        .map(|f| quote!(#f,))
        .collect();

    // Create the list of entries
    let mut entries = vec![String::new(); size];
    for s in strings {
        let primary_hash = str_hash(&s);
        let secondary_fn = secondary_hash_functions[primary_hash as usize % size];
        let secondary_hash = int_hash(primary_hash ^ secondary_fn) as usize % size;
        entries[secondary_hash] = s;
    }

    let entries_tokens: TokenStream2 = entries
        .into_iter()
        .map(|s| quote!(::perfect_hash::Entry::new(#s),))
        .collect();

    let ident = input_data.ident;
    quote!(
        const #ident: ::perfect_hash::PerfectHashTable<#size> = ::perfect_hash::PerfectHashTable::new(
            [#second_hash_functions_tokens],
           [#entries_tokens],
        );
    )
    .into()
}

fn hash_fn_does_not_have_collisions(
    f: u32,
    primary_hashes: &[u32],
    occupied_indices: &mut [bool],
) -> bool {
    for primary_hash in primary_hashes {
        let secondary_hash = int_hash(primary_hash ^ f) as usize % occupied_indices.len();
        if occupied_indices[secondary_hash] {
            // The bucket is already occupied, => collision
            return false;
        } else {
            occupied_indices[secondary_hash] = true;
        }
    }

    // No collisions found
    true
}
