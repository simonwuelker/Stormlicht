from .. import util
import os
import json
import subprocess

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

    # Build the testrunner
    util.run_cmd(["cargo", "build", "--bin=html5lib-testrunner"])

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

                        p = util.run_cmd(
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