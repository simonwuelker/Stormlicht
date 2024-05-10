from .. import util, log
import subprocess
import webbrowser


def test_font_rendering(args, unknown_args):
    # Build the testrunner
    log.info("Building font-rendering testrunner")
    util.Command.create("cargo").with_arguments(["build", "--bin=text-rendering"]).run()

    # Execute the test suite
    util.Command.create("python3").with_arguments(
        [
            "tests/text-rendering-tests/check.py",
            "--engine=target/debug/text-rendering",
            "--output=target/text-rendering-tests.html",
            "--testcases=tests/text-rendering-tests/testcases",
            "--font=tests/text-rendering-tests/fonts",
        ]
    ).run(timeout=30)

    if args.open:
        # Open the results in the default webbrowser
        webbrowser.open("target/text-rendering-tests.html")
