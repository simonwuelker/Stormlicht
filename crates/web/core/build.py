import sys
from jinja2 import Environment, FileSystemLoader
import json
import os
import pathlib

SIDES = ["top", "right", "bottom", "left"]

def to_camel_case(text):
    text = text.replace("-", " ").replace("_", " ")
    words = text.split()
    return "".join(word.capitalize() for word in words)

def to_snake_case(text):
    return text.replace("-", "_")

def build_named_entities(env, target_dir, download_dir):
    with open(download_dir / "html_named_entities.json", "r") as infile:
        named_entities = json.load(infile)

    template = env.get_template("named_entities.rs.jinja")
    autogenerated_code = template.render(named_entities=named_entities)

    with open(target_dir / "named_entities.rs", "w") as outfile:
        outfile.write(autogenerated_code)


def build_identifiers(env, target_dir):
    with open("identifiers.json", "r") as infile:
        identifiers = json.load(infile)

    template = env.get_template("identifiers.rs.jinja")
    autogenerated_code = template.render(identifiers=identifiers)

    with open(target_dir / "identifiers.rs", "w") as outfile:
        outfile.write(autogenerated_code)

def build_properties(env, target_dir):
    with open("properties.json", "r") as infile:
        properties = json.load(infile)
    
    # Build properties.rs
    template = env.get_template("properties.rs.jinja")
    autogenerated_code = template.render(properties=properties, to_camel_case=to_camel_case, SIDES=SIDES)

    with open(target_dir / "properties.rs", "w") as outfile:
        outfile.write(autogenerated_code)

    # Build computed_style.rs
    inherited_properties = [p for p in properties if p["inherited"]]
    non_inherited_properties = [p for p in properties if not p["inherited"]]

    template = env.get_template("computed_style.rs.jinja")
    autogenerated_code = template.render(
        inherited_properties=inherited_properties, 
        non_inherited_properties=non_inherited_properties, 
        to_snake_case=to_snake_case,
        to_camel_case=to_camel_case,
        SIDES=SIDES)
    # print(autogenerated_code)
    # exit(1)

    with open(target_dir / "computed_style.rs", "w") as outfile:
        outfile.write(autogenerated_code)



if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: {sys.argv[0]} [OUT_DIR] [DOWNLOAD_DIR]")
        exit(1)

    target_dir = pathlib.Path(sys.argv[1])
    download_dir = pathlib.Path(sys.argv[2])
    env = Environment(loader=FileSystemLoader("templates"))

    build_named_entities(env, target_dir, download_dir)
    build_identifiers(env, target_dir)
    build_properties(env, target_dir)
