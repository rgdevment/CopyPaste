#!/usr/bin/env python3
import pathlib
import re
import sys

SPARED = {
    "cp-mac-sys": "an FFI wrapper: a mutated extern call does not compile, so every mutant is unviable",
    "cp-win-sys": "an FFI wrapper, the same",
}
WHERE = pathlib.Path(".github/workflows/mutants.yml")
SWEEP = pathlib.Path(".github/workflows/mutants-sweep.yml")
TOLD = pathlib.Path(".cargo/mutants.toml")


def of_the_workspace() -> set[str]:
    return {at.name for at in pathlib.Path("crates").iterdir() if (at / "Cargo.toml").is_file()}


def named_in(at: pathlib.Path) -> set[str]:
    if not at.is_file():
        return set()
    return set(re.findall(r"-p (cp-[a-z-]+)", at.read_text(encoding="utf-8")))


def swallowed() -> dict[str, str]:
    if not TOLD.is_file():
        return {}
    said = TOLD.read_text(encoding="utf-8")
    globs = re.findall(r'"(crates/[^"]+)"', said)
    whole = {}
    for one in globs:
        parts = one.split("/")
        if len(parts) < 3:
            continue
        rest = "/".join(parts[2:])
        if rest in ("**", "*", "src", "src/**", "src/*"):
            whole[parts[1]] = one
    return whole


def main() -> int:
    every = of_the_workspace()
    covered = named_in(WHERE) | named_in(SWEEP)
    hidden = swallowed()
    status = 0

    for one, glob in sorted(hidden.items()):
        print(
            f"::error file={TOLD.as_posix()}::«{glob}» takes the whole of {one} out of mutation. "
            "A crate is left out in the matrix, where it is seen, not in a glob"
        )
        status = 1
    covered -= set(hidden)

    for one in sorted(every - covered - set(SPARED)):
        print(
            f"::error file={WHERE.as_posix()}::{one} is not in any mutation shard and not spared. "
            f"Add it to the matrix, or write in scripts/mutants_cover.py why its mutants say nothing"
        )
        status = 1

    for one in sorted(set(SPARED) & covered):
        print(
            f"::error file=scripts/mutants_cover.py::{one} is both spared and in a shard; "
            "take it out of SPARED"
        )
        status = 1

    for one in sorted(set(SPARED) - every):
        print(f"::error file=scripts/mutants_cover.py::{one} is spared and no longer exists")
        status = 1

    for one in sorted(covered - every):
        print(f"::error file={WHERE.as_posix()}::{one} is in a shard and is not a crate here")
        status = 1

    if status == 0:
        print(f"{len(covered)} crates mutated, {len(SPARED)} spared, none forgotten")
    return status


if __name__ == "__main__":
    raise SystemExit(main())
