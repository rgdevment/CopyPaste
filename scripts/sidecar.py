#!/usr/bin/env python3
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WHERE = ROOT / "app" / "src-tauri" / "binaries"


def asked_for() -> str:
    if "--target" in sys.argv:
        return sys.argv[sys.argv.index("--target") + 1]
    said = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True)
    for line in said.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ").strip()
    raise SystemExit("rustc no dijo para qué máquina compila")


def main() -> int:
    release = "--debug" not in sys.argv
    triple = asked_for()
    crossing = "--target" in sys.argv
    build = ["cargo", "build", "-p", "cp-panel", "--locked"]
    if release:
        build.append("--release")
    if crossing:
        build += ["--target", triple]
    subprocess.run(build, cwd=ROOT, check=True)

    tail = ".exe" if triple.endswith("-msvc") or triple.endswith("-gnu") else ""
    where = ROOT / "target"
    if crossing:
        where = where / triple
    made = where / ("release" if release else "debug") / f"cp-panel{tail}"
    if not made.exists():
        raise SystemExit(f"no se encontró {made}")

    WHERE.mkdir(parents=True, exist_ok=True)
    named = f"cp-panel-{triple}{tail}"
    for landed in (WHERE / named, made.parent / named):
        shutil.copy2(made, landed)
        print(f"{landed.relative_to(ROOT)} listo desde {made.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
