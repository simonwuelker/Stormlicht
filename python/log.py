import os
import platform

HEADER = "\033[95m"
BLUE = "\033[94m"
CYAN = "\033[96m"
GREEN = "\033[92m"
ORANGE = "\033[93m"
RED = "\033[91m"
ENDC = "\033[0m"
BOLD = "\033[1m"
UNDERLINE = "\033[4m"

# Enable colors on windows
if platform.system() == "Windows":
    os.system("color")

def colored(text: str, color: str) -> str:
    return color + text + ENDC

def bold(text: str) -> str:
    return BOLD + text + ENDC

def underline(text: str) -> str:
    return UNDERLINE + text + str

def error(msg: str, **kwargs):
    print("[" + colored("ERROR", RED + BOLD) + "]: " + msg, **kwargs)
    exit(1)

def info(msg: str, **kwargs):
    print("[" + colored("INFO", BLUE + BOLD) + "]: " + msg, **kwargs)

def warning(msg: str, **kwargs):
    print("[" + colored("WARNING", ORANGE + BOLD) + "]: " + msg, **kwargs)
