import json
import os
import pathlib
import sys

CAUGHT = ("Killed", "Timeout")
MISSED = ("Survived", "NoCoverage")


def told(one: pathlib.Path) -> dict | None:
    try:
        seen = json.loads(one.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as why:
        print(f"::error::{one} is not a report: {why}")
        return None
    if not isinstance(seen, dict):
        print(f"::error::{one} is not a report: it holds {type(seen).__name__}, not an object")
        return None
    return seen


def of_crates(one: pathlib.Path) -> tuple[int, int] | None:
    seen = told(one)
    if seen is None:
        return None
    try:
        return (
            int(seen.get("caught", 0)) + int(seen.get("timeout", 0)),
            int(seen.get("missed", 0)),
        )
    except (TypeError, ValueError) as why:
        print(f"::error::{one} counts nothing a number can hold: {why}")
        return None


def of_the_window(one: pathlib.Path) -> tuple[int, int] | None:
    seen = told(one)
    if seen is None:
        return None
    files = seen.get("files")
    if not isinstance(files, dict):
        print(f"::error::{one} is not a Stryker report: it names no files")
        return None
    caught = missed = 0
    for held in files.values():
        for mutant in (held or {}).get("mutants") or []:
            said = (mutant or {}).get("status")
            if said in CAUGHT:
                caught += 1
            elif said in MISSED:
                missed += 1
    return caught, missed


def score() -> int:
    root = pathlib.Path(os.environ.get("FROM", "outcomes"))
    want = int(os.environ.get("WANT", "1") or "0")
    want_window = int(os.environ.get("WANT_WINDOW", "0") or "0")
    badge = pathlib.Path(os.environ.get("BADGE", "mutants.json"))

    crates = sorted(root.rglob("outcomes.json"))
    window = sorted(root.rglob("stryker.json"))
    if len(crates) < want:
        print(
            f"::error::{len(crates)} of {want} crate shards reached here: "
            "a score over part of a sweep would be a lie"
        )
        return 1
    if len(window) < want_window:
        print(
            f"::error::{len(window)} of {want_window} window shards reached here: "
            "a score over part of a sweep would be a lie"
        )
        return 1

    caught = 0
    missed = 0
    for one in crates:
        said = of_crates(one)
        if said is None:
            return 1
        caught += said[0]
        missed += said[1]
    for one in window:
        said = of_the_window(one)
        if said is None:
            return 1
        caught += said[0]
        missed += said[1]

    total = caught + missed
    if total == 0:
        print("::error::not one mutant was tested, so there is no score to tell")
        return 1

    rate = caught * 100 / total
    shown = f"{rate:.1f}"
    colour = "green" if float(shown) >= 80 else "orange" if float(shown) >= 60 else "red"
    badge.write_text(
        json.dumps(
            {
                "schemaVersion": 1,
                "label": "mutants",
                "message": f"{shown}%",
                "color": colour,
            }
        ),
        encoding="utf-8",
    )

    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    lines = [
        "### What the sweep came to",
        "",
        "| shards | caught | survived | score |",
        "| ---: | ---: | ---: | ---: |",
        f"| {len(crates) + len(window)} | {caught} | {missed} | {shown}% |",
        "",
    ]
    if summary:
        with open(summary, "a", encoding="utf-8") as out:
            out.write("\n".join(lines) + "\n")
    else:
        print("\n".join(lines))

    print(shown)
    return 0


if __name__ == "__main__":
    sys.exit(score())
