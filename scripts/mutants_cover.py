#!/usr/bin/env python3
import pathlib
import re
import sys

SPARED = {
    "cp-mac-sys": "an FFI wrapper: a mutated extern call does not compile, so every mutant is unviable",
    "cp-win-sys": "an FFI wrapper, the same",
}
GATE = pathlib.Path(".github/workflows/mutants.yml")
SWEEP = pathlib.Path(".github/workflows/mutants-sweep.yml")
TOLD = pathlib.Path(".cargo/mutants.toml")
WHOLE = ("**", "*", "src", "src/**", "src/*", "src/*.rs", "src/**/*.rs", "")


def of_the_workspace() -> set[str]:
    return {at.name for at in pathlib.Path("crates").iterdir() if (at / "Cargo.toml").is_file()}


def named_in(at: pathlib.Path) -> set[str]:
    if not at.is_file():
        return set()
    return set(re.findall(r"-p (cp-[a-z-]+)", at.read_text(encoding="utf-8")))


def swallowed(every: set[str]) -> dict[str, str]:
    if not TOLD.is_file():
        return {}
    whole = {}
    for one in re.findall(r'"([^"]+)"', TOLD.read_text(encoding="utf-8")):
        said = one.strip().strip("/")
        if not said.startswith("crates/"):
            continue
        parts = said.split("/", 2)
        name = parts[1]
        rest = parts[2] if len(parts) > 2 else ""
        if name == "*":
            for each in every:
                whole[each] = one
            continue
        if name in every and rest in WHOLE:
            whole[name] = one
    return whole


def main() -> int:
    every = of_the_workspace()
    swept = named_in(SWEEP)
    gated = named_in(GATE)
    hidden = swallowed(every)
    status = 0

    for one, glob in sorted(hidden.items()):
        print(
            f"::error file={TOLD.as_posix()}::«{glob}» takes the whole of {one} out of mutation. "
            "A crate is left out in the matrix, where it is seen, not in a glob"
        )
        status = 1
    swept -= set(hidden)
    gated -= set(hidden)

    for one in sorted(every - swept - set(SPARED)):
        where = " It is in the branch gate, which only mutates the lines a branch changed." if one in gated else ""
        print(
            f"::error file={SWEEP.as_posix()}::{one} is in no shard of the sweep and is not spared.{where} "
            f"Add it, or write in {pathlib.Path(__file__).name} why its mutants say nothing"
        )
        status = 1

    for one in sorted(swept - gated - set(SPARED)):
        print(
            f"::warning file={GATE.as_posix()}::{one} is swept every week but no branch is checked against it"
        )

    for one in sorted(set(SPARED) & (swept | gated)):
        print(
            f"::error file={pathlib.Path(__file__).name}::{one} is both spared and in a shard; "
            "take it out of SPARED"
        )
        status = 1

    for one in sorted(set(SPARED) - every):
        print(f"::error file={pathlib.Path(__file__).name}::{one} is spared and no longer exists")
        status = 1

    for one in sorted((swept | gated) - every):
        print(f"::error file={SWEEP.as_posix()}::{one} is in a shard and is not a crate here")
        status = 1

    if status == 0:
        print(f"{len(swept)} crates swept, {len(gated)} gated on a branch, {len(SPARED)} spared")
    return status


if __name__ == "__main__":
    raise SystemExit(main())
