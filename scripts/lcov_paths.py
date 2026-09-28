#!/usr/bin/env python3
import pathlib
import sys

ROOT = pathlib.Path.cwd().resolve()


def under(said: str) -> str | None:
    said = said.strip().replace("\\", "/")
    whole = pathlib.Path(said)
    if not whole.is_absolute():
        return said
    try:
        return whole.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return None


def mend(at: pathlib.Path, lead: str) -> int:
    lines, held, left = [], 0, 0
    for line in at.read_text(encoding="utf-8").splitlines():
        if line.startswith("SF:"):
            said = under(line[3:])
            if said is None:
                left += 1
            else:
                if lead and not said.startswith(lead):
                    said = lead + said
                line = f"SF:{said}"
                held += 1
        lines.append(line)
    at.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    if held == 0:
        print(f"::error file={at.as_posix()}::{at.as_posix()} carries no source it could place")
        return -1
    if left:
        print(f"::warning file={at.as_posix()}::{left} of its paths fall outside the workspace and were left alone")
    print(f"{at.as_posix()}: {held} files")
    return held


def main() -> int:
    asked = sys.argv[1:]
    if not asked:
        print("which lcov files? each one as «path» or «path=prefix»", file=sys.stderr)
        return 2
    status = 0
    for one in asked:
        said, _, lead = one.partition("=")
        at = pathlib.Path(said)
        if not at.is_file():
            print(f"::error::{said} is not here")
            return 1
        if lead and not lead.endswith("/"):
            lead += "/"
        if mend(at, lead) < 0:
            status = 1
    return status


if __name__ == "__main__":
    raise SystemExit(main())
