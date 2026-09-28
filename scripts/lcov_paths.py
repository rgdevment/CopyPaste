#!/usr/bin/env python3
import pathlib
import sys

ROOT = pathlib.Path.cwd().resolve()


def under(said: str) -> str:
    said = said.strip().replace("\\", "/")
    whole = pathlib.Path(said)
    if not whole.is_absolute():
        return said
    try:
        return whole.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return said


def lead_of(at: pathlib.Path) -> str:
    try:
        inside = at.resolve().parent.relative_to(ROOT).as_posix()
    except ValueError:
        return ""
    first = inside.split("/")[0]
    return f"{first}/" if first not in (".", "") else ""


def main() -> int:
    if len(sys.argv) < 2:
        print("which lcov files?", file=sys.stderr)
        return 2
    for name in sys.argv[1:]:
        at = pathlib.Path(name)
        if not at.is_file():
            print(f"{name} is not here", file=sys.stderr)
            return 1
        lead = lead_of(at)
        lines = []
        for line in at.read_text(encoding="utf-8").splitlines():
            if line.startswith("SF:"):
                said = under(line[3:])
                if lead and not said.startswith(lead):
                    said = lead + said
                line = f"SF:{said}"
            lines.append(line)
        at.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"{name}: {sum(1 for one in lines if one.startswith('SF:'))} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
