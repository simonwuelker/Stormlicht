#!/bin/python3

import argparse
import subprocess
import os
import shutil

from . import log, util, downloads
from .testrunners import test_html_parser, test_font_rendering

def ensure_submodules_are_downloaded():
    util.Command.create("git").with_arguments(["submodule", "update", "--init", "--recursive"]).run()

def build_documentation(args, unknown_args):
    cmd = util.Command.create("cargo").with_arguments(["doc"]).with_forwarded_arguments(unknown_args)

    if args.open:
        cmd.append_argument("--open")

    log.info("Building stormlicht documentation...")
    cmd.run()

def clean(args, unknown_args):
    cmd = util.Command.create("cargo").with_arguments(["clean"]).with_forwarded_arguments(unknown_args)

    log.info("Removing target directory...")
    cmd.run()


def run_stormlicht(args, unknown_args):
    if args.chrome == "gtk":
        build_gtk_blueprints()

    cmd = util.Command.create("cargo").with_arguments(["run"]).with_forwarded_arguments(unknown_args)

    if args.release:
        cmd.append_argument("--release")

    log.info("Compiling and running stormlicht...")
    cmd.run()


def build_stormlicht(args, unknown_args):
    build_gtk_blueprints()

    cmd = util.Command.create("cargo").with_arguments(["build"]).with_forwarded_arguments(unknown_args)

    if args.release:
        cmd.append_argument("--release")

    log.info("Compiling stormlicht...")
    cmd.run()


def build_gtk_blueprints():
    log.info("Compiling GTK blueprints...")

    blueprint_dir = "stormlicht/resources"
    blueprint_files = [
        os.path.join(blueprint_dir, file)
        for file in os.listdir(blueprint_dir)
        if file.endswith(".blp")
    ]

    util.Command.create("blueprint-compiler").with_arguments(["batch-compile", blueprint_dir, blueprint_dir]).extend_arguments(blueprint_files).run()

def test_stormlicht(args, unknown_args):
    cmd = util.Command.create("cargo").with_arguments(["test"]).with_forwarded_arguments(unknown_args).run()

def run():
    # Install git pre-commit hook
    log.info("Installing git commit hook...")
    shutil.copy("hooks/pre-commit.hook", ".git/hooks/pre-commit")

    log.info("Downloading submodules if necessary...")
    ensure_submodules_are_downloaded()

    log.info("Downloading required files...")
    downloads.download_required_files()

    # Main parser
    parser = argparse.ArgumentParser(
        prog="Stormlicht",
        description="Build system for the Stormlicht browser engine",
        epilog="Thank you for playing around with Stormlicht!",
    )

    subparsers = parser.add_subparsers(required=True)

    # Remove build files
    parser_clean = subparsers.add_parser("clean", help="Remove target directory")
    parser_clean.add_argument(
        "--open",
        action="store_true",
        help="Open documentation in the browser after building",
    )
    parser_clean.set_defaults(handler=clean)

    # Documentation
    parser_doc = subparsers.add_parser("doc", help="Build documentation")
    parser_doc.add_argument(
        "--open",
        action="store_true",
        help="Open documentation in the browser after building",
    )
    parser_doc.set_defaults(handler=build_documentation)

    # Run browser
    parser_run = subparsers.add_parser("run", help="Run Stormlicht")
    parser_run.add_argument(
        "--release",
        action="store_true",
        help="Build in release mode",
    )
    parser_run.add_argument(
        "--chrome",
        choices=["glazier", "gtk"],
        default="glazier",
        help="Which browser chrome to use"
    )
    parser_run.set_defaults(handler=run_stormlicht)

    # Build browser
    parser_build = subparsers.add_parser("build", help="Build Stormlicht")
    parser_build.add_argument(
        "--release",
        action="store_true",
        help="Build in release mode",
    )
    parser_build.set_defaults(handler=build_stormlicht)

    # Testing
    parser_test = subparsers.add_parser("test", help="Test Stormlicht")
    parser_test.set_defaults(handler=test_stormlicht)

    test_subparsers = parser_test.add_subparsers()
    parser_test_text_rendering = test_subparsers.add_parser(
        "text-rendering", help="Test text rendering"
    )
    parser_test_text_rendering.add_argument(
        "--open",
        action="store_true",
        help="Open text rendering report in the browser after completion",
    )
    parser_test_text_rendering.set_defaults(handler=test_font_rendering)

    parser_test_html = test_subparsers.add_parser("html", help="Test html parsing")
    parser_test_html.add_argument(
        "filter", nargs="?", default=None, help="Filter test cases by name"
    )
    parser_test_html.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Print detailed information about each failed test",
    )
    parser_test_html.set_defaults(handler=test_html_parser)

    args, unknown_args = parser.parse_known_args()
    args.handler(args, unknown_args)
