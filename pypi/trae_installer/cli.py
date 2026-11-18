import os
import subprocess
import sys


def _bin_path(exe_name: str) -> str:
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "bin", exe_name)


def main_cc() -> None:
    exe = _bin_path("forgecc.exe")
    if not os.path.isfile(exe):
        print(f"Error: {exe} not found.", file=sys.stderr)
        sys.exit(1)
    res = subprocess.run([exe, *sys.argv[1:]])
    sys.exit(res.returncode)


def main_studio() -> None:
    exe = _bin_path("forge-studio.exe")
    if not os.path.isfile(exe):
        print(f"Error: {exe} not found.", file=sys.stderr)
        sys.exit(1)
    subprocess.Popen([exe, *sys.argv[1:]])
