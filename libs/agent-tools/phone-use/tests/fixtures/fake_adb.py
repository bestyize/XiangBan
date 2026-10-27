"""Local protocol simulator. Never connects to ADB or any device, and never executes supplied shell text."""
import os
import re
import sys
import time

# Emulate the byte stream of Android's non-PTY shell, including on a Windows test host.
sys.stdout.reconfigure(encoding="utf-8", newline="\n")

log_path = sys.argv[sys.argv.index("--log") + 1]
serial = sys.argv[sys.argv.index("-s") + 1]


def log(message):
    with open(log_path, "a", encoding="utf-8") as file:
        file.write(message + "\n")


log("start " + serial)
for line in sys.stdin:
    marker = re.search(r"'(xb-[a-f0-9]+-[0-9]+):BEGIN'", line)
    if marker is None:
        sys.exit(5)
    marker = marker.group(1)
    command = line.split("; ( ", 1)[1].rsplit(" ) 2>&1;", 1)[0]
    log("run " + serial + " " + command)
    sys.stdout.write(marker + ":BEGIN\n")
    sys.stdout.flush()
    if command == "DROP":
        sys.stdout.write("partial effect\n")
        sys.stdout.flush()
        os._exit(7)
    if command == "SLEEP":
        time.sleep(10)
    output = {"OK": "ok", "EMPTY": "", "MULTI": "alpha\nbeta\n", "BIG": "x" * 4096,
              "MARKERLIKE": "xb-other:END:0\nkept", "FAIL": "simulated command failure"}.get(command, command)
    code = 9 if command == "FAIL" else 0
    sys.stdout.write(output + "\n" + marker + ":END:" + str(code) + "\n")
    sys.stdout.flush()
