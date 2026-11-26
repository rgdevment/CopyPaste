#!/usr/bin/env python3
import ctypes
import os
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WHERE = ROOT / "app" / "src-tauri" / "binaries"
VSWHERE = pathlib.Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)")) / (
    r"Microsoft Visual Studio\Installer\vswhere.exe"
)
# Windows itself never carries these; a runner (or a developer) that has them in System32 got them from vc_redist.
VC_RUNTIME = re.compile(r"^((msvcp|msvcr|vcruntime|concrt|vccorlib|vcomp|vcamp|mfc)\d|ucrtbased)", re.IGNORECASE)
REDIST_ARCH = {"x86_64": "x64"}


def asked_for() -> str:
    if "--target" in sys.argv:
        return sys.argv[sys.argv.index("--target") + 1]
    said = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True)
    for line in said.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ").strip()
    raise SystemExit("rustc did not say which machine it compiles for")


def visual_studio() -> pathlib.Path:
    if not VSWHERE.exists():
        raise SystemExit(f"{VSWHERE} was not found: the Visual C++ runtime the panel needs comes from Visual Studio")
    said = subprocess.run(
        [str(VSWHERE), "-latest", "-products", "*", "-requires",
         "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"],
        capture_output=True, text=True, check=True,
    )
    if not said.stdout.strip():
        raise SystemExit("vswhere found no Visual Studio with the C++ tools")
    return pathlib.Path(said.stdout.strip().splitlines()[0])


def default_version(studio: pathlib.Path, which: str) -> str:
    said = studio / "VC" / "Auxiliary" / "Build" / f"Microsoft.{which}Version.default.txt"
    if not said.exists():
        raise SystemExit(f"{said} was not found: install the C++ build tools with their redistributable files")
    return said.read_text().strip()


def redist_folder(studio: pathlib.Path, arch: str) -> pathlib.Path:
    given = os.environ.get("VCToolsRedistDir")
    base = pathlib.Path(given) if given else studio / "VC" / "Redist" / "MSVC" / default_version(studio, "VCRedist")
    held = sorted((base / arch).glob("Microsoft.VC*.CRT"))
    if not held:
        raise SystemExit(f"no Microsoft.VC*.CRT folder under {base / arch}: install the C++ redistributable files")
    return held[-1]


def dumpbin(studio: pathlib.Path) -> pathlib.Path:
    given = os.environ.get("VCToolsInstallDir")
    tools = pathlib.Path(given) if given else studio / "VC" / "Tools" / "MSVC" / default_version(studio, "VCTools")
    found = tools / "bin" / "Hostx64" / "x64" / "dumpbin.exe"
    if not found.exists():
        raise SystemExit(f"{found} was not found")
    return found


def imports(tool: pathlib.Path, binary: pathlib.Path) -> list[str]:
    said = subprocess.run([str(tool), "/nologo", "/dependents", str(binary)], capture_output=True, text=True, check=True)
    names = (line.strip() for line in said.stdout.splitlines() if line.startswith("    "))
    return [name for name in names if " " not in name and name.lower().endswith(".dll")]


def windows_provides(dll: str) -> bool:
    if dll.lower().startswith(("api-ms-win-", "ext-ms-")):
        return True
    system = pathlib.Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32"
    return not VC_RUNTIME.match(dll) and (system / dll).exists()


def runtime_needed(panel: pathlib.Path, redist: pathlib.Path, tool: pathlib.Path) -> list[pathlib.Path]:
    carried: dict[str, pathlib.Path] = {}
    pending = [panel]
    while pending:
        binary = pending.pop()
        for dll in imports(tool, binary):
            if dll.lower() in carried:
                continue
            beside = redist / dll.lower()
            if beside.exists():
                carried[dll.lower()] = beside
                pending.append(beside)
            elif not windows_provides(dll):
                raise SystemExit(f"{binary.name} needs {dll}, which neither Windows nor {redist} provides")
    return sorted(carried.values())


def file_version(path: pathlib.Path) -> str:
    version = ctypes.windll.version
    size = version.GetFileVersionInfoSizeW(str(path), None)
    data = ctypes.create_string_buffer(size)
    fixed = ctypes.c_void_p()
    length = ctypes.c_uint()
    if not (size and version.GetFileVersionInfoW(str(path), 0, size, data)
            and version.VerQueryValueW(data, "\\", ctypes.byref(fixed), ctypes.byref(length))):
        raise SystemExit(f"{path} carries no version")
    ms, ls = ctypes.cast(fixed, ctypes.POINTER(ctypes.c_uint32))[2:4]
    return f"{ms >> 16}.{ms & 0xFFFF}.{ls >> 16}.{ls & 0xFFFF}"


def lay_runtime_beside(panel: pathlib.Path, triple: str) -> None:
    for stale in WHERE.glob("*.dll"):
        stale.unlink()
    if not triple.endswith("-windows-msvc"):
        return
    arch = REDIST_ARCH.get(triple.split("-")[0])
    if arch is None:
        raise SystemExit(f"no Visual C++ runtime folder is known for {triple}")
    studio = visual_studio()
    redist = redist_folder(studio, arch)
    for dll in runtime_needed(panel, redist, dumpbin(studio)):
        shutil.copy2(dll, WHERE / dll.name)
        print(f"{(WHERE / dll.name).relative_to(ROOT)} {file_version(dll)} ready from {redist}")


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
        raise SystemExit(f"{made} was not found")

    WHERE.mkdir(parents=True, exist_ok=True)
    named = f"cp-panel-{triple}{tail}"
    for landed in (WHERE / named, made.parent / named):
        shutil.copy2(made, landed)
        print(f"{landed.relative_to(ROOT)} ready from {made.relative_to(ROOT)}")
    lay_runtime_beside(made, triple)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
