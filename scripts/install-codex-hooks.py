#!/usr/bin/env python3

import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description="Install apm's automatic Codex hooks")
    parser.add_argument("--store-url", help="Clone the store from this Git URL if needed")
    args = parser.parse_args()

    repo = Path(__file__).resolve().parent.parent
    home = Path(os.environ.get("CODEX_HOME") or Path.home() / ".codex").resolve()
    data = Path(os.environ.get("XDG_DATA_HOME") or Path.home() / ".local" / "share")
    store = data / "apm" / "claudemds"
    if not (store / ".git").is_dir():
        if args.store_url is None:
            parser.error(f"no Git-backed apm store at {store}; supply --store-url")
        store.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "clone", "--", args.store_url, str(store)], check=True)

    subprocess.run(
        ["cargo", "build", "--locked", "--target-dir", str(repo / "target")],
        cwd=repo,
        check=True,
    )

    path = home / "hooks.json"
    home.mkdir(parents=True, exist_ok=True)
    config = json.loads(path.read_text()) if path.exists() else {}
    hooks = config.setdefault("hooks", {})
    command = shlex.join([sys.executable, str(repo / "scripts" / "codex-apm-hook.py")])
    previous = "python3 " + shlex.quote(str(home / "hooks" / "codex-apm-hook.py"))
    # Preserve other hooks and the trust of an existing apm installation.
    for event, matcher, message in (
        ("SessionStart", "^(startup|resume)$", "Pulling project instructions with apm"),
        ("Stop", None, "Syncing updated project instructions with apm"),
    ):
        groups = hooks.setdefault(event, [])
        if any(
            handler.get("command") in (command, previous)
            for group in groups
            for handler in group.get("hooks", [])
        ):
            continue
        group = {"hooks": [{
            "type": "command",
            "command": command,
            "timeout": 45,
            "statusMessage": message,
        }]}
        if matcher:
            group["matcher"] = matcher
        groups.append(group)

    if path.is_symlink():
        path = path.resolve()
    new_contents = json.dumps(config, indent=2, ensure_ascii=False) + "\n"
    if not path.exists() or path.read_text() != new_contents:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=path.parent, delete=False
        ) as output:
            temp = Path(output.name)
            output.write(new_contents)
        try:
            if path.exists():
                temp.chmod(path.stat().st_mode & 0o777)
            temp.replace(path)
        finally:
            temp.unlink(missing_ok=True)

    print(f"Configured Codex hooks in {path}")
    print("Open Codex and review and trust the apm hooks with /hooks.")


if __name__ == "__main__":
    main()
