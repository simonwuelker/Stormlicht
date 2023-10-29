from . import log
import subprocess

def run_cmd(cmd, stdin=None, stdout=None, stderr=None, timeout=None):
    result = subprocess.run(cmd, stdin, stdout, stderr, timeout)
    if result.returncode != 0:
        log.error(f"Failed to run {cmd}: Process exited with exit code {result.returncode}")