from .. import util, log
import os
import json
import subprocess


def test_html_parser(args, unknown_args):
    def verbose_print(initial_state, testdata, out, stderr):
        print(log.bold("Initial State") + ":", initial_state)
        print(
            log.bold("Input") + ":        ",
            testdata["input"].encode("unicode_escape").decode("utf-8"),
        )
        print(log.bold("Expected") + ":     ", testdata["output"])
        print(log.bold("Got") + ":          ", out)
        print(log.bold("stderr") + ":       ", stderr)
        print()

    # Build the testrunner
    util.Command.create("cargo").with_arguments(
        ["build", "--bin=html5lib-testrunner"]
    ).run()

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

                    runner_args = [
                        '--state="{}"'.format(initial_state),
                        '--input="{}"'.format(
                            test["input"].encode("unicode_escape").decode("utf-8")
                        ),
                    ]

                    if "lastStartTag" in test:
                        runner_args.append(
                            '--last-start-tag="{}"'.format(test["lastStartTag"])
                        )

                    p = (
                        util.Command.create("./target/debug/html5lib-testrunner")
                        .with_arguments(runner_args)
                        .run(
                            ignore_failure=True,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE,
                            timeout=3,
                        )
                    )

                    try:
                        out_text = p.stdout.decode("utf-8")
                        out = json.loads(out_text)

                        # if "doubleEscaped" is set, then the output was unicode-escaped twice and
                        # we need to unescape it once more
                        if "doubleEscaped" in test and test["doubleEscaped"]:
                            for token_index in range(len(test["output"])):
                                for element_index in range(
                                    len(test["output"][token_index])
                                ):
                                    test["output"][token_index][element_index] = (
                                        test["output"][token_index][element_index]
                                        .encode("ascii")
                                        .decode("unicode_escape")
                                    )

                    except subprocess.TimeoutExpired:
                        print(log.colored("Timed out", log.RED))
                        tests_failed += 1
                        if args.verbose:
                            verbose_print(initial_state, test, "", "")
                        continue
                    except Exception as e:
                        print(log.colored("Fail (Exception)", log.RED))
                        tests_failed += 1

                        if args.verbose:
                            verbose_print(initial_state, test, p.stdout, p.stderr)
                        continue

                    if out == test["output"]:
                        print(log.colored("Success", log.GREEN))
                    else:
                        print(log.colored("Fail", log.RED))
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
