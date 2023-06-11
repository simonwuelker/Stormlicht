#!/bin/python3

import argparse
import subprocess
import webbrowser
import os
import json


def ensure_submodules_are_downloaded():
    subprocess.run(["git", "submodule", "update", "--init", "--recursive"])


def build_documentation(args):
    cmd = ["cargo", "doc"]
    if args.open:
        cmd.append("--open")
    subprocess.run(cmd)


def run(args):
    cmd = ["cargo", "run"]
    if args.release:
        cmd.append("--release")
    subprocess.run(cmd)


def build(args):
    cmd = ["cargo", "build"]
    if args.release:
        cmd.append("--release")
    subprocess.run(cmd)


def test(args):
    ensure_submodules_are_downloaded()

    cmd = ["cargo", "t"]
    subprocess.run(cmd)


def test_font_rendering(args):
    ensure_submodules_are_downloaded()

    # Build the testrunner
    subprocess.run(["cargo", "build", "--bin=text-rendering"])

    # Execute the test suite
    subprocess.run(
        [
            "python3",
            "tests/text-rendering-tests/check.py",
            "--engine=target/debug/text-rendering",
            "--output=target/text-rendering-tests.html",
            "--testcases=tests/text-rendering-tests/testcases",
            "--font=tests/text-rendering-tests/fonts",
        ]
    )

    if args.open:
        # Open the results in the default webbrowser
        webbrowser.open("target/text-rendering-tests.html")


def test_html_parser(args):
    ensure_submodules_are_downloaded()

    # Build the testrunner
    subprocess.run(["cargo", "build", "--bin=html5lib-testrunner"])

    total_tests = 0
    tests_failed = 0
    for test_name in os.listdir("tests/html5lib-tests/tokenizer"):
        if test_name.endswith(".test") and test_name != "xmlViolation.test":
            with open(
                os.path.join("tests/html5lib-tests/tokenizer", test_name), "r"
            ) as testfile:
                testdata = json.load(testfile)
            print(test_name)
            for test in testdata["tests"]:
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
                        p = subprocess.run(
                            [
                                "./target/debug/html5lib-testrunner",
                                '--state="{}"'.format(initial_state),
                                '--input="{}"'.format(test["input"]),
                                "--testcases=tests/text-rendering-tests/testcases",
                                "--font=tests/text-rendering-tests/fonts",
                            ],
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE,
                        )

                        out = json.loads(p.stdout.decode("utf-8"))
                    except:
                        print("Fail")
                        tests_failed += 1
                        continue

                    if out == test["output"]:
                        print("Success")
                    else:
                        print("Fail")
                        tests_failed += 1

    print()
    print(
        f"{tests_failed}/{total_tests} failed ({tests_failed/total_tests * 100:.2f}%)"
    )


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

parser_test_text_rendering = test_subparsers.add_parser(
    "html", help="test html parsing"
)
parser_test_text_rendering.set_defaults(handler=test_html_parser)

args = parser.parse_args()
args.handler(args)
