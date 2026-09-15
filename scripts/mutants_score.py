import json
import os
import pathlib
import sys


def score() -> int:
    root = pathlib.Path(os.environ.get("FROM", "outcomes"))
    want = int(os.environ.get("WANT", "1"))
    badge = pathlib.Path(os.environ.get("BADGE", "mutants.json"))

    found = sorted(root.rglob("outcomes.json"))
    if len(found) < want:
        print(
            f"::error::{len(found)} of {want} shards reached here: "
            "a score over part of a sweep would be a lie"
        )
        return 1

    caught = 0
    missed = 0
    for one in found:
        try:
            seen = json.loads(one.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as why:
            print(f"::error::{one} is not a report: {why}")
            return 1
        caught += int(seen.get("caught", 0)) + int(seen.get("timeout", 0))
        missed += int(seen.get("missed", 0))

    total = caught + missed
    if total == 0:
        print("::error::not one mutant was tested, so there is no score to tell")
        return 1

    rate = caught * 100 / total
    colour = "green" if rate >= 80 else "orange" if rate >= 60 else "red"
    badge.write_text(
        json.dumps(
            {
                "schemaVersion": 1,
                "label": "mutants",
                "message": f"{rate:.1f}%",
                "color": colour,
            }
        ),
        encoding="utf-8",
    )

    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    lines = [
        "### What the sweep came to",
        "",
        f"| shards | caught | survived | score |",
        "| ---: | ---: | ---: | ---: |",
        f"| {len(found)} | {caught} | {missed} | {rate:.1f}% |",
        "",
    ]
    if summary:
        with open(summary, "a", encoding="utf-8") as out:
            out.write("\n".join(lines) + "\n")
    else:
        print("\n".join(lines))

    print(f"{rate:.1f}")
    return 0


if __name__ == "__main__":
    sys.exit(score())
