#!/usr/bin/env python3
import json
import os
import re
import subprocess
import sys
from datetime import datetime

MAIN = "refs/heads/main"
TAGS = "refs/heads/refs/tags/"
DRY = os.environ.get("DRY_RUN", "").lower() == "true"
RUST = re.compile(r"^(v0-rust-.+)-[0-9a-f]{8}-[0-9a-f]{8}$")
GONE = "Could not find a cache matching"


def gh(*args):
    done = subprocess.run(["gh", *args], capture_output=True, text=True)
    if done.returncode != 0:
        sys.exit(f"gh {' '.join(args)} failed: {done.stderr.strip()}")
    return done.stdout


def listed(ref=None):
    args = ["cache", "list", "--limit", "100", "--json", "key,ref,sizeInBytes,createdAt"]
    if ref:
        args += ["--ref", ref]
    return json.loads(gh(*args))


def superseded():
    households = {}
    for one in listed(MAIN):
        found = RUST.match(one["key"])
        if found:
            households.setdefault(found.group(1), []).append({**one, "ref": MAIN})
    stale = []
    for household in households.values():
        household.sort(key=lambda one: datetime.fromisoformat(one["createdAt"]), reverse=True)
        stale.extend(household[1:])
    return stale


def petrified():
    return [one for one in listed() if one["ref"].startswith(TAGS)]


def main():
    stale = superseded() + petrified()
    if not stale:
        print("nothing superseded and no tag left anything behind")
        return

    freed = 0
    failed = []
    for one in stale:
        print(f"  {one['sizeInBytes'] // 1048576:>5} MB  {one['ref']}  {one['key']}")
        if DRY:
            freed += one["sizeInBytes"]
            continue
        done = subprocess.run(
            ["gh", "cache", "delete", one["key"], "--ref", one["ref"]],
            capture_output=True,
            text=True,
        )
        if done.returncode == 0:
            freed += one["sizeInBytes"]
        elif GONE in done.stderr:
            print(f"         evicted before we got to it: {one['key']}")
        else:
            failed.append(f"{one['key']}: {done.stderr.strip()}")

    said = "would sweep" if DRY else "swept"
    print(f"::notice::{said} {len(stale) - len(failed)} cache(s), {freed // 1048576} MB")
    if failed:
        sys.exit("::error::" + "; ".join(failed))


if __name__ == "__main__":
    main()
