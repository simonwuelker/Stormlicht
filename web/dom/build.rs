use std::{
    collections::{HashMap, HashSet},
    env, fs, io,
    path::Path,
};

use proc_macro2::{Delimiter, TokenTree};

#[derive(Default, Debug)]
struct TypeJournal {
    /// Contains all the types that should be modeled
    types: HashSet<String>,
    /// Mapping from parent type -> types that inherit from it
    children: HashMap<String, Vec<String>>,
    /// The structs that don't inherit from anything
    roots: Vec<String>,
}

fn search_for_derived_struct_in_file<P: AsRef<Path>>(
    path: P,
    type_journal: &mut TypeJournal,
) -> Result<(), io::Error> {
    let file_contents = fs::read_to_string(path)?;
    let ast = syn::parse_file(&file_contents).unwrap();

    for item in ast.items {
        if let syn::Item::Struct(struct_def) = item {
            // Check if the struct defines an "inherit" attribute
            let mut inherits_from = None;
            for attr in struct_def.attrs {
                if let Some(ident) = attr.path.get_ident() {
                    if ident == "inherit" {
                        if attr.tokens.is_empty() {
                            // This is a root object, it does not inherit from anything but the type system
                            // still needs to know about it
                            let root_name = struct_def.ident.to_string();
                            if !type_journal.types.insert(root_name.clone()) {
                                panic!("{root_name:?} was declared twice");
                            }
                            type_journal.roots.push(root_name);

                            break;
                        }

                        let mut tokens = attr.tokens.into_iter();
                        let group = match tokens.next() {
                            Some(TokenTree::Group(group))
                                if group.delimiter() == Delimiter::Parenthesis =>
                            {
                                group
                            },
                            _ => panic!("Invalid inherit attribute"),
                        };

                        let mut argument_tokens = group.stream().into_iter();
                        let parent_type_name = match argument_tokens.next() {
                            Some(TokenTree::Ident(ident)) => ident,

                            _ => panic!("Invalid inherit attribute"),
                        };
                        if argument_tokens.next().is_some() {
                            panic!("Invalid inherit attribute, are you trying to specify a full path (like foo::bar)? Because that's not allowed.")
                        }

                        inherits_from = Some(parent_type_name.to_string());
                        break;
                    }
                }
            }

            if let Some(parent_name) = inherits_from {
                let struct_name = struct_def.ident.to_string();
                if !type_journal.types.insert(struct_name.clone()) {
                    panic!("{struct_name:?} was declared twice");
                }

                type_journal
                    .children
                    .entry(parent_name)
                    .or_default()
                    .push(struct_name);
            }
        }
    }

    Ok(())
}

fn build_inheritance_enum(parent: &str, children: &[String], hierarchy: &TypeJournal) -> String {
    let variants = children
        .iter()
        .map(|child| {
            if hierarchy.children.contains_key(child) {
                // This child has sub-children, so we need to store its InheritanceObject
                format!("{child}({child}InheritanceObject)")
            } else {
                child.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(",");

    format!(
        "
        #[derive(Clone, Copy, Debug)]
        pub enum {parent}InheritanceObject {{
            {variants}
        }}
        "
    )
}

fn find_path(
    from_parents: &[String],
    want: &str,
    type_journal: &TypeJournal,
) -> Option<Vec<String>> {
    for parent in from_parents {
        if parent == want {
            return Some(vec![parent.clone()]);
        }

        if let Some(children) = type_journal.children.get(parent) {
            if let Some(mut path) = find_path(children, want, type_journal) {
                path.push(parent.clone());
                return Some(path);
            }
        }
    }
    None
}

fn format_path(path: &[String], type_journal: &TypeJournal) -> String {
    let mut result = "GlobalInheritanceObject".to_string();
    let mut open_parenthesis = 0;

    for typename in path.iter().skip(1).rev() {
        result.push_str(&format!("::{typename}({typename}InheritanceObject"));
        open_parenthesis += 1;
    }

    // The very last object in the chain might not have any children.
    // In that case, we should not open another pair of parenthesis
    let first_object_in_path = path.first().unwrap();
    result.push_str(&format!("::{first_object_in_path}"));
    if type_journal.children.get(first_object_in_path).is_some() {
        result.push_str("(_)");
    }

    // close the corrent number of parenthesis
    result.push_str(&")".repeat(open_parenthesis));

    result
}

fn main() -> Result<(), io::Error> {
    // Rerun if any DOM object changes
    // TODO: Since this is probably going to take a considerable amount of time
    // if the number of DOM object grows, we should consider caching and only updating
    // the files that changed.
    println!("cargo:rerun-if-changed=src/dom_objects");

    // Used to keep track of who derives from where
    let mut type_journal = TypeJournal::default();

    // Search for inherited structs in each file inside src/dom_objects
    for dir_entry_or_error in fs::read_dir("src/dom_objects")? {
        let dir_entry = dir_entry_or_error?;

        if dir_entry.file_type()?.is_file() {
            search_for_derived_struct_in_file(dir_entry.path(), &mut type_journal)?;
        } else {
            println!("cargo:warning=Found directory {}, files in subdirectories are NOT considered by the inheritance system!", dir_entry.path().display())
        }
    }

    // Generate the required enums & trait impls
    let typenames = type_journal
        .types
        .iter()
        .cloned()
        .collect::<Vec<String>>()
        .join(",");

    let local_inheritance_trees = type_journal
        .children
        .iter()
        .map(|(parent, children)| build_inheritance_enum(parent, children, &type_journal))
        .collect::<Vec<String>>()
        .join("");

    let global_inheritance_tree =
        build_inheritance_enum("Global", &type_journal.roots, &type_journal);

    let inheritance_trait_impls = type_journal
        .types
        .iter()
        .map(|typename| {
            format!(
                "
                impl DOMTyped for crate::dom_objects::{typename} {{
                    fn as_type() -> DOMType {{
                        DOMType::{typename}
                    }}
                }}
        "
            )
        })
        .collect::<Vec<String>>()
        .join("");

    let castable_trait_impl_cases = type_journal
        .types
        .iter()
        .map(|typename| {
            if let Some(path) = find_path(&type_journal.roots, typename, &type_journal) {
                format!(
                    "DOMType::{typename} => matches!(self.underlying_type(), {})",
                    format_path(&path, &type_journal)
                )
            } else {
                panic!("{typename} could not be reached from any root")
            }
        })
        .collect::<Vec<String>>()
        .join(",");

    let autogenerated_code = format!(
        "
        {global_inheritance_tree}
        {local_inheritance_trees}

        #[derive(Clone, Copy, Debug)]
        pub enum DOMType {{
            {typenames}
        }}

        trait DOMTyped {{
            fn as_type() -> DOMType;
        }}
        {inheritance_trait_impls}

        trait Castable {{
            fn is_a<T: DOMTyped>(&self) -> bool;
        }}

        impl<P> Castable for crate::DOMPtr<P> {{
            fn is_a<T: DOMTyped>(&self) -> bool {{
                match T::as_type() {{
                    {castable_trait_impl_cases}
                }}
            }}
        }}
        "
    );

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("inheritance_autogenerated.rs");
    fs::write(dest_path, autogenerated_code)?;

    Ok(())
}
