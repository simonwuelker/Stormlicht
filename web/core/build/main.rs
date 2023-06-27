mod inheritance;
mod named_entities;

pub fn main() {
    inheritance::generate().expect("Generating inheritance code failed");
    named_entities::generate().expect("Generating html named entities failed");
}
