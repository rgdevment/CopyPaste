#!/usr/bin/env python3
import pathlib
import re
import sys

A_FILE = 1500
A_FUNCTION = 250
KEPT = pathlib.Path(".github/oversized.txt")
SAID = KEPT.as_posix()
WHERE = ("crates", "app/src", "app/src-tauri/src")
TABLES = ("locales.ts",)
QUOTED = re.compile(r'"(?:[^"\\]|\\.)*"')
CHARRED = re.compile(r"'(?:[^'\\]|\\.)'")
NOTED = re.compile(r"//.*$")
ONLY_IN_TESTS = re.compile(r"#\[cfg\((all\()?\s*test\b")
HEADED = re.compile(
    r"(?P<lead>\s*)(pub(\([^)]*\))? )?(default )?(const )?(async )?(unsafe )?"
    r'(extern "[A-Za-z-]+" )?fn (?P<name>\w+)'
)


def bare_of(line: str) -> str:
    return NOTED.sub("", CHARRED.sub("''", QUOTED.sub('""', line)))


def braces(line: str) -> int:
    bare = bare_of(line)
    return bare.count("{") - bare.count("}")


def block_ends(lines: list[str], from_turn: int) -> int:
    last = len(lines)
    ahead = from_turn
    while ahead < last and not lines[ahead].strip():
        ahead += 1
    depth, opened = 0, False
    while ahead < last:
        bare = bare_of(lines[ahead])
        depth += braces(lines[ahead])
        opened = opened or "{" in bare
        if opened and depth <= 0:
            break
        if not opened and bare.rstrip().endswith(";"):
            break
        ahead += 1
    return ahead


def blanked(at: pathlib.Path) -> list[str | None]:
    lines = at.read_text(encoding="utf-8", errors="replace").splitlines()
    kept: list[str | None] = list(lines)
    turn, last = 0, len(lines)
    while turn < last:
        if not ONLY_IN_TESTS.match(lines[turn].strip()):
            turn += 1
            continue
        ends = block_ends(lines, turn + 1)
        for each in range(turn, min(ends + 1, last)):
            kept[each] = None
        turn = ends + 1
    return kept


def code_of(at: pathlib.Path) -> int:
    if at.suffix != ".rs":
        return len(at.read_text(encoding="utf-8", errors="replace").splitlines())
    return sum(1 for line in blanked(at) if line is not None)


def held_of(at: pathlib.Path) -> dict[str, int]:
    if at.suffix != ".rs":
        return {}
    lines = blanked(at)
    found: dict[str, int] = {}
    open_at: list[tuple[str, int, int]] = []
    waiting: tuple[str, int] | None = None
    depth = 0
    for turn, line in enumerate(lines):
        if line is None:
            continue
        bare = bare_of(line)
        if waiting is None:
            said = HEADED.match(line)
            if said:
                waiting = (said.group("name"), turn)
        if waiting is not None:
            if "{" in bare:
                open_at.append((waiting[0], waiting[1], depth))
                waiting = None
            elif ";" in bare:
                waiting = None
        for letter in bare:
            if letter == "{":
                depth += 1
            elif letter == "}":
                depth -= 1
                while open_at and depth <= open_at[-1][2]:
                    name, began, _ = open_at.pop()
                    found[name] = max(found.get(name, 0), turn - began + 1)
    for name, began, _ in open_at:
        found[name] = max(found.get(name, 0), len(lines) - began)
    return found


def reaching() -> list[pathlib.Path]:
    every = []
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
            if where == "crates" and "/src/" not in said and "/examples/" not in said:
                continue
            every.append(at)
    return every


def measured() -> dict[str, int]:
    found = {}
    for at in reaching():
        said = at.as_posix()
        found[said] = code_of(at)
        for name, held in held_of(at).items():
            found[f"{said}::{name}"] = held
    return found


def roof_of(what: str) -> int:
    return A_FUNCTION if "::" in what else A_FILE


def noted() -> dict[str, int] | None:
    if not KEPT.is_file():
        return {}
    kept = {}
    for turn, line in enumerate(KEPT.read_text(encoding="utf-8").splitlines(), 1):
        said = line.strip()
        if not said or said.startswith("#"):
            continue
        parts = said.split(None, 2)
        if len(parts) < 3 or not parts[0].isdigit():
            print(f"::error file={SAID},line={turn}::{SAID} wants a count, a path and why it stays, and this line says «{said}»")
            return None
        kept[parts[1]] = int(parts[0])
    return kept


def main() -> int:
    found = measured()
    if "--print" in sys.argv:
        for what, held in sorted(found.items(), key=lambda one: -one[1]):
            print(f"{held} {what}")
        return 0
    kept = noted()
    if kept is None:
        return 1
    status = 0
    for what, now in sorted(found.items()):
        roof = roof_of(what)
        was = kept.get(what)
        at, _, name = what.partition("::")
        it = f"{name} in {at}" if name else at
        if was is None:
            if now > roof:
                print(f"::error file={at}::{it} is {now} lines of code; {roof} is the ceiling. Split it along a seam, or write «{now} {what} why it stays» into {SAID}")
                status = 1
        elif now > was:
            print(f"::error file={at}::{it} was already over the ceiling at {was} lines and grew to {now}. What is above it only shrinks")
            status = 1
    for what in kept:
        if what not in found:
            print(f"::error file={SAID}::{what} is in {SAID} and is not there any more; take the line out")
            status = 1
    return status


if __name__ == "__main__":
    raise SystemExit(main())
