import os
import platform

HEADER = "\033[95m"
BLUE = "\033[94m"
CYAN = "\033[96m"
GREEN = "\033[92m"
WARNING = "\033[93m"
FAIL = "\033[91m"
ENDC = "\033[0m"
BOLD = "\033[1m"
UNDERLINE = "\033[4m"

# Enable colors on windows
if platform.system() == "Windows":
    os.system("color")

def error(msg):
    print("[" + FAIL + BOLD + "ERROR" + ENDC + "]: " + msg)
    exit(1)

def info(msg):
    print("[" + BLUE + BOLD + "INFO" + ENDC + "]: " + msg)

def warning(msg):
    print("[" + WARNING + BOLD + "WARNING" + ENDC + "]: " + msg)
