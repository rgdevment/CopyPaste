#!/usr/bin/env python3
import pathlib
import re
import sys

CEILING = 1500
KEPT = pathlib.Path(".github/oversized.txt")
SAID = KEPT.as_posix()
WHERE = ("crates", "app/src", "app/src-tauri/src")
TABLES = ("locales.ts",)
QUOTED = re.compile(r'"(?:[^"\\]|\\.)*"')


def braces(line: str) -> int:
    bare = QUOTED.sub('""', line)
    return bare.count("{") - bare.count("}")


def code_of(at: pathlib.Path) -> int:
    lines = at.read_text(encoding="utf-8", errors="replace").splitlines()
    if at.suffix != ".rs":
        return len(lines)
    held, turn, last = 0, 0, len(lines)
    while turn < last:
        if lines[turn].strip() == "#[cfg(test)]":
            ahead = turn + 1
            while ahead < last and not lines[ahead].strip():
                ahead += 1
            if ahead < last and lines[ahead].lstrip().startswith("mod "):
                depth, opened = 0, False
                while ahead < last:
                    depth += braces(lines[ahead])
                    opened = opened or "{" in QUOTED.sub('""', lines[ahead])
                    if opened and depth <= 0:
                        break
                    ahead += 1
                turn = ahead + 1
                continue
        held += 1
        turn += 1
    return held


def measured() -> dict[str, int]:
    found = {}
    for where in WHERE:
        base = pathlib.Path(where)
        if not base.is_dir():
            continue
        for at in base.rglob("*"):
            if at.suffix not in (".rs", ".ts", ".tsx"):
                continue
            said = at.as_posix()
            if "node_modules" in said or "/tests/" in said or ".test." in said:
                continue
            if any(said.endswith(one) for one in TABLES):
                continue
            if where == "crates" and "/src/" not in said:
                continue
            found[said] = code_of(at)
    return found


def noted() -> dict[str, int] | None:
    if not KEPT.is_file():
        return {}
    kept = {}
    for turn, line in enumerate(KEPT.read_text(encoding="utf-8").splitlines(), 1):
        said = line.strip()
        if not said or said.startswith("#"):
            continue
        parts = said.split(None, 1)
        if len(parts) != 2 or not parts[0].isdigit():
            print(f"::error file={SAID},line={turn}::{SAID} wants a count and a path, and this line says «{said}»")
            return None
        kept[parts[1].split("  ")[0].strip()] = int(parts[0])
    return kept


def main() -> int:
    found = measured()
    if "--print" in sys.argv:
        for at, held in sorted(found.items(), key=lambda one: -one[1]):
            print(f"{held} {at}")
        return 0
    kept = noted()
    if kept is None:
        return 1
    status = 0
    for at, now in sorted(found.items()):
        was = kept.get(at)
        if was is None:
            if now > CEILING:
                print(f"::error file={at}::{at} is {now} lines of code; {CEILING} is the ceiling. Split it along a seam, or write «{now} {at}  why it stays» into {SAID}")
                status = 1
        elif now > was:
            print(f"::error file={at}::{at} was already over the ceiling at {was} lines and grew to {now}. What is above it only shrinks")
            status = 1
    for at in kept:
        if at not in found:
            print(f"::error file={SAID}::{at} is in {SAID} and no longer exists; take the line out")
            status = 1
    return status


if __name__ == "__main__":
    raise SystemExit(main())
