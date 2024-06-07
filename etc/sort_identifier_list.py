"""
Sorts the identifiers in web/identifiers.json alphabetically
"""

import json

IDENTIFIERS_PATH = "crates/web/identifiers.json"

with open(IDENTIFIERS_PATH, "r") as infile:
    identifiers = json.load(infile)

identifiers = sorted(identifiers)

with open(IDENTIFIERS_PATH, "w") as outfile:
    json.dump(identifiers, outfile, indent=4)
