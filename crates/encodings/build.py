from jinja2 import Environment, FileSystemLoader
import json
import pathlib
import sys

REPLACEMENT = 0xFFFF

def transform_encoding_name(name):
    return name.replace("-", "_")

def map_to_char(values):
    # We concat the strings inside the python code since its really slow in flask
    return "[" + ",".join([rf"'\u{{{code:x}}}'" if code is not None else r"'\u{FFFD}'" for code in values]) + "]"

def build_encodings(env, target_dir, download_dir):
    with open(download_dir / "encodings.json", "r") as infile:
        encodings = json.load(infile)

    # Merge the encoding blocks together 
    encodings = sum((block["encodings"] for block in encodings), [])

    template = env.get_template("encodings.rs.jinja")
    autogenerated_code = template.render(encodings=encodings)

    with open(target_dir / "encodings.rs", "w") as outfile:
        outfile.write(autogenerated_code)

def build_indexes(env, target_dir, download_dir):
    with open(download_dir / "indexes.json", "r") as infile:
        indexes = json.load(infile)

    template = env.get_template("indexes.rs.jinja")
    autogenerated_code = template.render(indexes=indexes)

    with open(target_dir / "indexes.rs", "w") as outfile:
        outfile.write(autogenerated_code)

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: {sys.argv[0]} [OUT_DIR] [DOWNLOAD_DIR]")
        exit(1)

    target_dir = pathlib.Path(sys.argv[1])
    download_dir = pathlib.Path(sys.argv[2])
    env = Environment(loader=FileSystemLoader("templates"))
    env.filters["transform_encoding_name"] = transform_encoding_name
    env.filters["map_to_char"] = map_to_char

    build_encodings(env, target_dir, download_dir)
    build_indexes(env, target_dir, download_dir)