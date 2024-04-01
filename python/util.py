from . import log
import shutil
import subprocess
import os

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

class Command:
    def create(name: str):
        cmd = Command()
        cmd.binary = ExecutableStore.get(name)
        cmd.args = []
        cmd.env = os.environ.copy()
        cmd.forwarded_args = []
        return cmd
    
    def with_arguments(self, args: list):
        self.args = args
        return self

    def with_environment(self, env: dict):
        self.env = env
        return self

    def with_forwarded_arguments(self, forwarded_args: list):
        self.forwarded_args = forwarded_args
        return self
    
    def append_argument(self, arg: str):
        self.args.append(arg)
        return self
    
    def extend_arguments(self, args: list):
        self.args += args
        return self
    
    def run(self, ignore_failure=False, **kwargs):
        cmd = [self.binary] + self.args

        if len(self.forwarded_args) != 0:
            cmd += ["--"] + self.forwarded_args

        result = subprocess.run(cmd, env=self.env, **kwargs)
        if result.returncode != 0 and not ignore_failure:
            log.error(f"Failed to run {cmd}: Process exited with exit code {result.returncode}")
        
        return result
