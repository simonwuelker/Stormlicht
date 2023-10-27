#!/bin/python3

import argparse
import subprocess
import webbrowser
import os
import json
import re


def ensure_submodules_are_downloaded():
    subprocess.run(["git", "submodule", "update", "--init", "--recursive"])


def build_documentation(args, unknown_args):
    cmd = ["cargo", "doc"]
    if args.open:
        cmd.append("--open")
    subprocess.run(cmd)


def run(args, unknown_args):
    build_gtk_blueprints()
    cmd = ["cargo", "run"]
    if args.release:
        cmd.append("--release")
    cmd.append("--")
    cmd.extend(unknown_args)
    subprocess.run(cmd)


def build(args, unknown_args):
    build_gtk_blueprints()
    cmd = ["cargo", "build"]
    if args.release:
        cmd.append("--release")
    subprocess.run(cmd)


def build_gtk_blueprints():
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
    subprocess.run(cmd)


def test(args, unknown_args):
    ensure_submodules_are_downloaded()

    cmd = ["cargo", "t"]
    subprocess.run(cmd)


def test_font_rendering(args, unknown_args):
    ensure_submodules_are_downloaded()

    # Build the testrunner
    subprocess.run(["cargo", "build", "--bin=text-rendering"], check=True)

    # Execute the test suite
    subprocess.run(
        [
            "python3",
            "tests/text-rendering-tests/check.py",
            "--engine=target/debug/text-rendering",
            "--output=target/text-rendering-tests.html",
            "--testcases=tests/text-rendering-tests/testcases",
            "--font=tests/text-rendering-tests/fonts",
        ],
        timeout=30,
    )

    if args.open:
        # Open the results in the default webbrowser
        webbrowser.open("target/text-rendering-tests.html")


def test_html_parser(args, unknown_args):
    def verbose_print(initial_state, testdata, out, stderr):
        print("Initial state:", initial_state)
        print(
            "Input:         '{}'".format(
                testdata["input"].encode("unicode_escape").decode("utf-8")
            )
        )
        print("Expected:       {}".format(testdata["output"]))
        print("Got:            {}".format(out))
        print(f"stderr:         {stderr}")
        print()

    ensure_submodules_are_downloaded()

    # Build the testrunner
    subprocess.run(["cargo", "build", "--bin=html5lib-testrunner"], check=True)

    total_tests = 0
    tests_failed = 0
    for test_name in os.listdir("tests/html5lib-tests/tokenizer"):
        if test_name.endswith(".test") and test_name != "xmlViolation.test":
            with open(
                os.path.join("tests/html5lib-tests/tokenizer", test_name), "r"
            ) as testfile:
                testdata = json.load(testfile)

            for test in testdata["tests"]:
                if args.filter != None:
                    if args.filter not in test["description"]:
                        continue

                if "initialStates" in test:
                    initial_states = test["initialStates"]
                else:
                    initial_states = ["Data state"]

                for initial_state in initial_states:
                    total_tests += 1

                    if len(initial_states) == 1:
                        print(f'Testing: {test["description"]} - ', end="")
                    else:
                        print(
                            f'Testing: {test["description"]}({initial_state}) - ',
                            end="",
                        )

                    try:
                        runner_args = [
                            "./target/debug/html5lib-testrunner",
                            '--state="{}"'.format(initial_state),
                            '--input="{}"'.format(test["input"]),
                        ]

                        if "lastStartTag" in test:
                            runner_args.append(
                                '--last-start-tag="{}"'.format(test["lastStartTag"])
                            )

                        p = subprocess.run(
                            runner_args,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE,
                            timeout=3,
                        )

                        out_text = p.stdout.decode("utf-8")
                        if "doubleEscaped" in test and test["doubleEscaped"]:
                            out_text = out_text.encode("unicode_escape").decode("utf-8")

                        out = json.loads(out_text)
                    except subprocess.TimeoutExpired:
                        print("Timed out")
                        tests_failed += 1
                        if args.verbose:
                            verbose_print(initial_state, test, "", "")
                        continue
                    except:
                        print("Fail")
                        tests_failed += 1

                        if args.verbose:
                            verbose_print(initial_state, test, p.stdout, p.stderr)
                        continue
                    if out == test["output"]:
                        print("Success")
                    else:
                        print("Fail")
                        tests_failed += 1

                        if args.verbose:
                            verbose_print(initial_state, test, out, p.stderr)

    print()
    if total_tests != 0:
        print(
            f"{tests_failed}/{total_tests} tests failed ({tests_failed/total_tests * 100:.2f}%)"
        )
    else:
        print("No tests were run.")


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
parser_run.set_defaults(handler=run)

# Build browser
parser_build = subparsers.add_parser("build", help="build Stormlicht")
parser_build.add_argument(
    "--release",
    action="store_true",
    help="Build in release mode",
)
parser_build.set_defaults(handler=build)

# Testing
parser_test = subparsers.add_parser("test", help="test Stormlicht")
parser_test.set_defaults(handler=test)

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
