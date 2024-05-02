#![feature(iter_intersperse)]

use std::{env, fs, io, path::Path};

use serialize::Deserialize;
use serialize_json::{JsonDeserializer, JsonError};

const OBJECT_IDENTIFIERS_PATH: &str = "object_identifiers.json";
const OBJECTS_XREF_PATH: &str = "objects_xref.json";

/// A Namespace as specified in the json file
#[derive(Deserialize, Clone, Debug)]
struct Namespace {
    digits: Vec<usize>,
    short_name: String,
    long_name: String,
    elements: Vec<Namespace>,
}

#[derive(Deserialize, Clone, Debug)]
struct ObjectCrossReference {
    name: String,
    digest: String,
    algorithm: String,
}

#[derive(Clone, Debug)]
struct ObjectIdentifier {
    digits: Vec<usize>,
    name: String,
}

#[derive(Debug)]
// Dead code analysis ignores debug impls (which are called when the error is returned from main)
#[allow(dead_code)]
enum Error {
    IO(io::Error),
    Deserialization(JsonError),
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn cleanup_identifier(ident: &str) -> String {
    capitalize(ident).replace([' ', '-', '.', '/', '_'], "")
}

impl ObjectCrossReference {
    fn as_match_arm(&self) -> String {
        let digest = if self.digest != "undefined" {
            format!("Some(Self::{})", cleanup_identifier(&self.digest))
        } else {
            "None".to_string()
        };

        format!(
            "Self::{base} => ReferencedAlgorithms {{ digest: {digest}, public_key_algorithm: Self::{pk_algorithm} }}", 
            base = cleanup_identifier(&self.name),
            pk_algorithm = cleanup_identifier(&self.algorithm)
        )
    }
}

impl Namespace {
    fn enum_variant_name(&self) -> String {
        let basename = if self
            .long_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            &self.long_name
        } else {
            &self.short_name
        };

        cleanup_identifier(basename)
    }

    fn get_identifiers(
        &self,
        current_path: &mut Vec<usize>,
        identifiers: &mut Vec<ObjectIdentifier>,
    ) {
        if self.digits.is_empty() {
            // This is an alias - add only the children
            for element in &self.elements {
                element.get_identifiers(current_path, identifiers);
            }
        } else {
            current_path.extend_from_slice(&self.digits);

            // Add the object identifer for this namespace
            let identifier = ObjectIdentifier {
                digits: current_path.clone(),
                name: self.enum_variant_name(),
            };
            identifiers.push(identifier);

            // Add all its children
            for element in &self.elements {
                element.get_identifiers(current_path, identifiers);
            }

            // Remove the digits we added again
            current_path.truncate(current_path.len() - self.digits.len());
        }
    }
}

fn main() -> Result<(), Error> {
    // NOTE: Thanks to the openssl developers for collecting so many different
    //       object identifiers and their meaning in https://github.com/openssl/openssl/blob/master/crypto/objects/objects.txt
    //       Our "object_identifiers.json" is a cleaned up version of their file.
    //       (This note can't be in the json itself because json doesn't support comments :/)

    println!("cargo:rerun-if-changed={OBJECT_IDENTIFIERS_PATH}");
    println!("cargo:rerun-if-changed={OBJECTS_XREF_PATH}");

    // Deserialize list of known object identifiers
    let json = String::from_utf8(fs::read(OBJECT_IDENTIFIERS_PATH)?)
        .expect("object_identifiers.json contains invalid utf-8");
    let root_namespaces: Vec<Namespace> = Vec::deserialize(&mut JsonDeserializer::new(&json))?;

    // Deserialize object references
    // These map an algorithm identifier to the digest and public key algorithm used
    let json = String::from_utf8(fs::read(OBJECTS_XREF_PATH)?)
        .expect("objects_xref.json contains invalid utf-8");
    let object_crossreferences: Vec<ObjectCrossReference> =
        Vec::deserialize(&mut JsonDeserializer::new(&json))?;

    // Create a constant for every known object identifier
    let mut object_identifiers = vec![];
    let mut path = vec![];
    for namespace in &root_namespaces {
        namespace.get_identifiers(&mut path, &mut object_identifiers);
    }

    let enum_variants: String = object_identifiers
        .iter()
        .map(|identifier| identifier.name.as_str())
        .intersperse(",")
        .collect();

    let match_arms: String = object_identifiers
        .iter()
        .map(|identifier| format!("{:?} => Self::{}", identifier.digits, identifier.name))
        .intersperse(",".to_string())
        .collect();

    let to_digit_arms: String = object_identifiers
        .iter()
        .map(|identifier| format!("Self::{} => &{:?}", identifier.name, identifier.digits))
        .intersperse(",".to_string())
        .collect();

    let reference_match_arms: String = object_crossreferences
        .iter()
        .map(|reference| reference.as_match_arm())
        .intersperse(",".to_string())
        .collect();

    let autogenerated_code = format!(
        "
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum ObjectIdentifier {{
            {enum_variants},
        }}

        #[derive(Clone, Copy, Debug)]
        pub struct ReferencedAlgorithms {{
            pub digest: Option<ObjectIdentifier>,
            pub public_key_algorithm: ObjectIdentifier,
        }}

        #[derive(Clone, Copy, Debug)]
        pub struct UnknownObjectIdentifier;

        impl ObjectIdentifier {{
            pub fn digits(&self) -> &'static [usize] {{
                match self {{
                    {to_digit_arms}
                }}
            }}

            pub fn references(&self) -> Option<ReferencedAlgorithms> {{
                let reference = match self {{
                    {reference_match_arms},
                    _ => return None,
                }};

                Some(reference)
            }}

            pub fn try_from_digits(value: &[usize]) -> Result<Self, UnknownObjectIdentifier> {{
                let identifier = match *value {{
                    {match_arms},
                    _ => {{
                        return Err(UnknownObjectIdentifier);
                    }}
                }};
    
                Ok(identifier)
            }}
        }}
        "
    );

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("object_identifier.rs");
    fs::write(dest_path, autogenerated_code)?;

    Ok(())
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        Self::Deserialization(value)
    }
}
