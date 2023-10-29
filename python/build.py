#!/bin/python3

import argparse
import subprocess
import os
import shutil

from . import log, util, downloads
from .testrunners import test_html_parser, test_font_rendering

def ensure_submodules_are_downloaded():
    util.run_cmd(["git", "submodule", "update", "--init", "--recursive"])

def build_documentation(args, unknown_args):
    cmd = ["cargo", "doc"]
    if args.open:
        cmd.append("--open")
    util.run_cmd(cmd)


def run_stormlicht(args, unknown_args):
    build_gtk_blueprints()

    cmd = ["cargo", "run"]
    if args.release:
        cmd.append("--release")
    cmd.append("--")
    cmd.extend(unknown_args)

    log.info("Compiling and running stormlicht...")
    util.run_cmd(cmd)


def build_stormlicht(args, unknown_args):
    build_gtk_blueprints()

    cmd = ["cargo", "build"]
    if args.release:
        cmd.append("--release")

    log.info("Compiling stormlicht...")
    util.run_cmd(cmd)


def build_gtk_blueprints():
    log.info("Compiling GTK blueprints...")

    blueprint_dir = "stormlicht/resources"
    blueprint_files = [
        os.path.join(blueprint_dir, file)
        for file in os.listdir(blueprint_dir)
        if file.endswith(".blp")
    ]
    cmd = [
        "blueprint-compiler",
        "batch-compile",
        blueprint_dir,
        blueprint_dir,
    ] + blueprint_files
    util.run_cmd(cmd)


def test_stormlicht(args, unknown_args):
    cmd = ["cargo", "t"]
    util.run_cmd(cmd)


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

    # Documentation
    parser_doc = subparsers.add_parser("doc", help="build documentation")
    parser_doc.add_argument(
        "--open",
        action="store_true",
        help="Open documentation in the browser after building",
    )
    parser_doc.set_defaults(handler=build_documentation)

    # Run browser
    parser_run = subparsers.add_parser("run", help="run Stormlicht")
    parser_run.add_argument(
        "--release",
        action="store_true",
        help="Build in release mode",
    )
    parser_run.set_defaults(handler=run_stormlicht)

    # Build browser
    parser_build = subparsers.add_parser("build", help="build Stormlicht")
    parser_build.add_argument(
        "--release",
        action="store_true",
        help="Build in release mode",
    )
    parser_build.set_defaults(handler=build_stormlicht)

    # Testing
    parser_test = subparsers.add_parser("test", help="test Stormlicht")
    parser_test.set_defaults(handler=test_stormlicht)

    test_subparsers = parser_test.add_subparsers()
    parser_test_text_rendering = test_subparsers.add_parser(
        "text-rendering", help="test text rendering"
    )
    parser_test_text_rendering.add_argument(
        "--open",
        action="store_true",
        help="Open text rendering report in the browser after completion",
    )
    parser_test_text_rendering.set_defaults(handler=test_font_rendering)

    parser_test_html = test_subparsers.add_parser("html", help="test html parsing")
    parser_test_html.add_argument(
        "filter", nargs="?", default=None, help="filter test cases by name"
    )
    parser_test_html.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="print detailed information about each failed test",
    )
    parser_test_html.set_defaults(handler=test_html_parser)

    args, unknown_args = parser.parse_known_args()
    args.handler(args, unknown_args)
