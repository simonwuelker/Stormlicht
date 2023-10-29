from . import log
import shutil
import subprocess

class ExecutableStore:
    known_executables = {}

    def get(name: str) -> str:
        if name in ExecutableStore.known_executables:
            return ExecutableStore.known_executables[name]
        else:
            log.info(f"Found '{name}': ", end="")
            full_path = shutil.which(name)

            if full_path is None:
                print(log.colored("NO", log.RED))
                exit(1)
            else:
                print(log.colored("YES", log.GREEN) + f" ({full_path})")
                ExecutableStore.known_executables[name] = full_path
                return full_path

def run_cmd(cmd, stdin=None, stdout=None, stderr=None, timeout=None):
    cmd[0] = ExecutableStore.get(cmd[0])
    result = subprocess.run(cmd, stdin, stdout, stderr, timeout)
    if result.returncode != 0:
        log.error(f"Failed to run {cmd}: Process exited with exit code {result.returncode}")