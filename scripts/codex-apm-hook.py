#!/usr/bin/env python3

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


def main():
    event = json.load(sys.stdin)
    action = {"SessionStart": "pull", "Stop": "push"}.get(event["hook_event_name"])
    if action is None:
        return {}

    # Prefer the binary built by the installer, with fallbacks for manual setups.
    binaries = [
        Path(__file__).resolve().parent.parent / "target" / "debug" / "apm",
        shutil.which("apm"),
        Path.home() / "projects" / "apm" / "target" / "debug" / "apm",
    ]
    cwd = event["cwd"]
    env = dict(os.environ, GIT_TERMINAL_PROMPT="0")
    for binary in binaries:
        if not binary or not os.access(binary, os.X_OK):
            continue
        help_result = subprocess.run(
            [binary, "store", "project", "--help"],
            cwd=cwd,
            env=env,
            capture_output=True,
            timeout=5,
        )
        if help_result.returncode == 0:
            break
    else:
        return {"systemMessage": "apm project sync requires a newer apm binary."}

    if action == "push":
        env["NOCHK"] = "1"
    result = subprocess.run(
        [binary, "store", "project", action],
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        errors="replace",
        timeout=35,
    )
    output = {}
    if result.returncode:
        message = result.stderr.strip() or f"exit code {result.returncode}"
        output["systemMessage"] = f"apm {action}: {message[:1200]}"
    if action == "pull" and result.stdout.strip():
        output["hookSpecificOutput"] = {
            "hookEventName": "SessionStart",
            "additionalContext": result.stdout,
        }
    return output


if __name__ == "__main__":
    try:
        print(json.dumps(main(), ensure_ascii=False))
    except (OSError, subprocess.TimeoutExpired, ValueError, KeyError) as error:
        print(json.dumps({"systemMessage": f"apm hook: {error}"}, ensure_ascii=False))
