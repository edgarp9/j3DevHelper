#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


class BuildReleaseError(RuntimeError):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build this Rust project in release mode and open the release artifact directory."
    )
    parser.add_argument(
        "--no-open",
        action="store_true",
        help="Build only; do not open the release artifact directory.",
    )
    return parser.parse_args()


def run_release_build(project_root: Path) -> None:
    cargo = shutil.which("cargo")
    if cargo is None:
        raise BuildReleaseError("cargo was not found in PATH.")

    command = [cargo, "build", "--release"]
    print(f"Running: {' '.join(command)}", flush=True)
    subprocess.run(command, cwd=project_root, check=True)


def open_directory(path: Path) -> None:
    directory = path.resolve()
    if not directory.is_dir():
        raise BuildReleaseError(f"release directory does not exist: {directory}")

    if sys.platform.startswith("win"):
        os.startfile(str(directory))  # type: ignore[attr-defined]
        return

    if sys.platform.startswith("linux"):
        for command in (["xdg-open"], ["gio", "open"], ["kde-open"], ["gnome-open"]):
            executable = shutil.which(command[0])
            if executable is None:
                continue
            subprocess.Popen(
                [executable, *command[1:], str(directory)],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            return
        raise BuildReleaseError(
            "no supported file manager opener was found. Install xdg-open or gio."
        )

    if sys.platform == "darwin":
        subprocess.Popen(
            ["open", str(directory)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return

    raise BuildReleaseError(f"unsupported platform for opening directories: {sys.platform}")


def main() -> int:
    args = parse_args()
    project_root = Path(__file__).resolve().parent
    release_dir = project_root / "target" / "release"

    try:
        run_release_build(project_root)
        print(f"Release directory: {release_dir}", flush=True)
        if not args.no_open:
            open_directory(release_dir)
            print("Opened the release directory.", flush=True)
    except subprocess.CalledProcessError as error:
        print(f"Release build failed with exit code {error.returncode}.", file=sys.stderr)
        return error.returncode if error.returncode else 1
    except BuildReleaseError as error:
        print(f"Release build failed: {error}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
