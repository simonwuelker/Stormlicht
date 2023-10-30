import requests
import os
import sys
import io
import zipfile
import subprocess

from . import log, util

BASE_DIR = "downloads"
FONTS_URL = "https://fonts.google.com/download?family=Roboto"
BROTLI_DICTIONARY_URL = (
    "https://github.com/google/brotli/raw/master/c/common/dictionary.bin"
)
BROTLI_REPO_URL = "https://github.com/google/brotli"
HTML_NAMED_ENTITIES_URL = "https://html.spec.whatwg.org/entities.json"

def download_required_files():
    def download(url):
        response = requests.get(url)
        if response.status_code != 200:
            log.error(
                f"Download failed with status code {response.status_code}: {response.text}"
            )
        return response.content
    
    if not os.path.exists(BASE_DIR):
        os.makedirs(BASE_DIR)

    if not os.path.exists(os.path.join(BASE_DIR, "html_named_entities.json")):
        log.info("Downloading html named entities list...")
        named_entities = download(HTML_NAMED_ENTITIES_URL)
        with open(os.path.join(BASE_DIR, "html_named_entities.json"), "wb") as f:
            f.write(named_entities)

    if not os.path.exists(os.path.join(BASE_DIR, "fonts/roboto/Roboto-Medium.ttf")):
        log.info("Downloading font files...")
        os.makedirs(os.path.join(BASE_DIR, "fonts/roboto"))
        zipped_fonts = zipfile.ZipFile(io.BytesIO(download(FONTS_URL)))
        zipped_fonts.extractall(path=os.path.join(BASE_DIR, "fonts/roboto"))

    if not os.path.exists(os.path.join(BASE_DIR, "brotli")):
        log.info("Downloading brotli dictionary...")
        os.makedirs(os.path.join(BASE_DIR, "brotli"))
        dictionary = download(BROTLI_DICTIONARY_URL)
        with open(os.path.join(BASE_DIR, "brotli/dictionary.bin"), "wb") as f:
            f.write(dictionary)

    if not os.path.exists(os.path.join(BASE_DIR, "brotli/testdata")):
        log.info("Downloading brotli testdata...")
        os.makedirs(os.path.join(BASE_DIR, "brotli/testdata"))
        testdata_dir = os.path.join(BASE_DIR, "brotli/testdata")

        util.Command.create("git").with_arguments([
                "clone",
                "-n",
                "--depth=1",
                "--filter=tree:0",
                "-q",
                BROTLI_REPO_URL,
                testdata_dir,
            ]).run(stderr=subprocess.DEVNULL)
        
        util.Command.create("git").with_arguments([
                "-C",
                testdata_dir,
                "sparse-checkout",
                "set",
                "tests/testdata",
                "-q",
            ]).run(stderr=subprocess.DEVNULL)
        
        util.Command.create("git").with_arguments([
                "-C",
                testdata_dir,
                "checkout",
                "-q",
            ]).run(stderr=subprocess.DEVNULL)
