import os
import sys

MARKS = {
    "@IDENTITY@": "MSIX_IDENTITY",
    "@PUBLISHER@": "MSIX_PUBLISHER",
    "@PUBLISHER_DISPLAY@": "MSIX_PUBLISHER_DISPLAY",
    "@VERSION@": "QUAD",
}
FALLBACKS = {"MSIX_PUBLISHER_DISPLAY": "RGDevment"}


def escaped(value: str) -> str:
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def filled(body: str, said: dict[str, str]) -> str:
    for mark, value in said.items():
        body = body.replace(mark, escaped(value))
    return body


def main(source: str, target: str) -> int:
    said = {}
    for mark, name in MARKS.items():
        value = os.environ.get(name) or FALLBACKS.get(name, "")
        if not value:
            print(f"::error::{name} is empty, and {mark} has nothing to become")
            return 1
        said[mark] = value

    with open(source, encoding="utf-8") as handle:
        body = filled(handle.read(), said)

    with open(target, "w", encoding="utf-8") as handle:
        handle.write(body)

    print(body)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
