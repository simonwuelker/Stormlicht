use std::{collections::HashMap, env, fmt::Write, fs, io, path::Path};

use serialize::{json::JsonDeserializer, Deserialize};

#[derive(Deserialize, Clone, Debug)]
struct CharacterEntity {
    #[allow(dead_code)] // Deserialization library doesn't allow us to ignore fields yet
    codepoints: Vec<u32>,
    characters: String,
}

const NAMED_ENTITIES: &str =
    include_str!(concat!(env!("DOWNLOAD_DIR"), "/html_named_entities.json"));

pub fn generate() -> Result<(), io::Error> {
    let mut deserializer = JsonDeserializer::new(NAMED_ENTITIES);
    let entities: HashMap<String, CharacterEntity> =
        HashMap::deserialize(&mut deserializer).expect("invalid named entities json");

    // TODO: There are smarter ways to generate this code, we just do a linear search over all
    // possible character references
    // Sort entity names by longest-first (since we always choose the longest match first)
    let mut names: Vec<&String> = entities.keys().collect();
    names.sort_unstable_by(|&a, &b| b.len().cmp(&a.len()));

    let if_blocks = names
        .iter()
        .map(|&name| {
            let characters = &entities.get(name).unwrap().characters.escape_default();
            // Our tokenizer already considered the leading ampersand
            let name = &name[1..];
            format!(
                "
            if html.starts_with(\"{name}\") {{
                return Some((\"{characters}\", \"{name}\"));
            }}
            "
            )
        })
        .reduce(|mut output, n| {
            let _ = write!(output, "{n}");
            output
        })
        .unwrap_or_default();

    let autogenerated_code = format!(
        "
        /// Returns a tuple of `(resolved_reference, matched_str)`
        pub fn lookup_character_reference(html: &str) -> Option<(&'static str, &'static str)> {{
            {if_blocks}
            None
        }}
        "
    );

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("named_entities.rs");
    fs::write(dest_path, autogenerated_code)?;

    Ok(())
}
