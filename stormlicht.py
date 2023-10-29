#!/bin/sh
# The beginning of this script is both valid shell and valid python,
# such that the script starts with the shell and is reexecuted with
# the right python.
# This idea came from https://github.com/servo/servo/blob/master/mach
''':' && if [ ! -z "$MSYSTEM" ] ; then exec python "$0" "$@" ; else which python3 > /dev/null 2> /dev/null && exec python3 "$0" "$@" || exec python "$0" "$@" ; fi
'''


from python.build import run

if __name__ == "__main__":
    run()