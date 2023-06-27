use std::{collections::HashMap, fs, io};

use serialize::{json::JsonDeserializer, Deserialize};

#[derive(Deserialize, Clone, Debug)]
struct CharacterEntity {
    codepoints: Vec<u32>,
    characters: String,
}

pub fn generate() -> Result<(), io::Error> {
    let json = String::from_utf8(fs::read("../../downloads/html_named_entities.json")?)
        .expect("html_named_entities.json contains invalid utf-8");
    let mut deserializer = JsonDeserializer::new(&json);
    let entities: HashMap<String, CharacterEntity> =
        HashMap::deserialize(&mut deserializer).expect("invalid named entities json");
    Ok(())
}
