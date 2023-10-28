#!/usr/bin/python3

"""
Adds a meson-wrap file with method=cargo
"""

import sys
import requests

CRATES_IO_URL = "https://crates.io/api/v1/crates/"

WRAP_FILE_TEMPLATE = """
[wrap-file]
directory = {}
source_url = {}
source_filename = {}
source_hash = {}
method = cargo
"""

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 {} <crate-name>".format(sys.argv[0]))
        exit(1)

    crate_name = sys.argv[1]
    metadata = requests.get(f"{CRATES_IO_URL}{crate_name}").json()

    # The first version is the most recent one
    crate_data = metadata["versions"][0]

    source_hash = crate_data["checksum"]
    source_url = "https://crates.io" + crate_data["dl_path"]
    version = crate_data["num"]
    directory = f"{crate_name}-{version}"
    source_filename = crate_name.replace("-", "_") + "-" + version + ".tar.gz"

    wrap_file = WRAP_FILE_TEMPLATE.format(directory, source_url, source_filename, source_hash)

    with open(f"subprojects/{crate_name}-rs.wrap", "w") as outfile:
        outfile.write(wrap_file)


