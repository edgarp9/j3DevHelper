#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import dataclass
from fnmatch import fnmatch
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import subprocess
import sys
import zipfile


NOTICE_FILE_NAMES = (
    "LICENSE",
    "THIRD_PARTY_NOTICES.txt",
    "about.txt",
)
BINARY_EXTRA_FILE_NAMES = (
    *NOTICE_FILE_NAMES,
    "README.md",
    "icon.svg",
    "icon.ico",
)
SOURCE_EXCLUDED_DIR_NAMES = {
    ".git",
    ".idea",
    ".my",
    ".vscode",
    "__pycache__",
    "coverage",
    "criterion",
    "dist",
    "target",
}
SOURCE_EXCLUDED_FILE_NAMES = {
    ".DS_Store",
    "Desktop.ini",
    "Thumbs.db",
    "cargo-tarpaulin-report.xml",
    "flamegraph.svg",
    "tarpaulin-report.html",
}
SOURCE_EXCLUDED_FILE_PATTERNS = (
    "*.bak",
    "*.ilk",
    "*.log",
    "*.pdb",
    "*.profdata",
    "*.profraw",
    "*.rlib",
    "*.rmeta",
    "*.swo",
    "*.swp",
    "*.tmp",
    "*~",
)


class BuildReleaseError(RuntimeError):
    pass


@dataclass(frozen=True)
class PackageMetadata:
    name: str
    version: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Build this Rust project in release mode and create source/binary "
            "release zip packages."
        )
    )
    parser.add_argument(
        "--no-open",
        action="store_true",
        help="Build and package only; do not open the release artifact directory.",
    )
    return parser.parse_args()


def read_package_metadata(project_root: Path) -> PackageMetadata:
    manifest = project_root / "Cargo.toml"
    name: str | None = None
    version: str | None = None
    in_package_section = False

    for raw_line in manifest.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("[") and line.endswith("]"):
            in_package_section = line == "[package]"
            continue
        if not in_package_section:
            continue

        name = name or parse_toml_string_value(line, "name")
        version = version or parse_toml_string_value(line, "version")
        if name is not None and version is not None:
            return PackageMetadata(name=name, version=version)

    raise BuildReleaseError("Cargo.toml [package] name/version could not be read.")


def parse_toml_string_value(line: str, key: str) -> str | None:
    prefix = f"{key} ="
    if not line.startswith(prefix):
        return None

    value = line.split("=", 1)[1].strip()
    if len(value) < 2 or not value.startswith('"') or not value.endswith('"'):
        return None
    return value[1:-1]


def run_release_build(project_root: Path) -> None:
    cargo = shutil.which("cargo")
    if cargo is None:
        raise BuildReleaseError("cargo was not found in PATH.")

    command = [cargo, "build", "--release"]
    print(f"Running: {' '.join(command)}", flush=True)
    subprocess.run(command, cwd=project_root, check=True)


def stage_notice_files(project_root: Path, release_dir: Path) -> None:
    for file_name in NOTICE_FILE_NAMES:
        source = project_root / file_name
        if not source.is_file():
            raise BuildReleaseError(f"required notice file is missing: {source}")
        shutil.copy2(source, release_dir / file_name)


def create_release_packages(
    project_root: Path,
    release_dir: Path,
    metadata: PackageMetadata,
) -> tuple[Path, Path]:
    dist_dir = release_dir / "dist"
    dist_dir.mkdir(parents=True, exist_ok=True)

    source_package = create_source_package(project_root, dist_dir, metadata)
    binary_package = create_binary_package(project_root, release_dir, dist_dir, metadata)
    return source_package, binary_package


def create_source_package(
    project_root: Path,
    dist_dir: Path,
    metadata: PackageMetadata,
) -> Path:
    package_path = dist_dir / f"{metadata.name}-v{metadata.version}-source.zip"
    root_prefix = f"{metadata.name}-v{metadata.version}-source"

    with zipfile.ZipFile(package_path, "w", zipfile.ZIP_DEFLATED) as archive:
        for path, relative_path in iter_source_files(project_root):
            archive.write(
                path,
                PurePosixPath(root_prefix, *relative_path.parts).as_posix(),
            )

    return package_path


def iter_source_files(project_root: Path) -> list[tuple[Path, Path]]:
    files: list[tuple[Path, Path]] = []
    for path in project_root.rglob("*"):
        relative_path = path.relative_to(project_root)
        if is_source_excluded(relative_path):
            continue
        if path.is_file():
            files.append((path, relative_path))
    return sorted(files, key=lambda item: item[1].as_posix())


def is_source_excluded(relative_path: Path) -> bool:
    if any(part in SOURCE_EXCLUDED_DIR_NAMES for part in relative_path.parts):
        return True
    if relative_path.name in SOURCE_EXCLUDED_FILE_NAMES:
        return True
    return any(fnmatch(relative_path.name, pattern) for pattern in SOURCE_EXCLUDED_FILE_PATTERNS)


def create_binary_package(
    project_root: Path,
    release_dir: Path,
    dist_dir: Path,
    metadata: PackageMetadata,
) -> Path:
    binary_name = release_binary_name(metadata)
    binary_path = release_dir / binary_name
    if not binary_path.is_file():
        raise BuildReleaseError(f"release binary was not found: {binary_path}")

    tag = platform_tag()
    package_path = dist_dir / f"{metadata.name}-v{metadata.version}-{tag}.zip"
    root_prefix = f"{metadata.name}-v{metadata.version}-{tag}"

    with zipfile.ZipFile(package_path, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.write(binary_path, PurePosixPath(root_prefix, binary_name).as_posix())
        for file_name in BINARY_EXTRA_FILE_NAMES:
            source = project_root / file_name
            if source.is_file():
                archive.write(source, PurePosixPath(root_prefix, file_name).as_posix())

    return package_path


def release_binary_name(metadata: PackageMetadata) -> str:
    if sys.platform.startswith("win"):
        return f"{metadata.name}.exe"
    return metadata.name


def platform_tag() -> str:
    if sys.platform.startswith("win"):
        os_tag = "windows"
    elif sys.platform.startswith("linux"):
        os_tag = "linux"
    elif sys.platform == "darwin":
        os_tag = "macos"
    else:
        os_tag = sys.platform.replace(os.sep, "-")

    machine = platform.machine().lower() or "unknown"
    machine = machine.replace("amd64", "x86_64")
    return f"{os_tag}-{machine}"


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
        metadata = read_package_metadata(project_root)
        run_release_build(project_root)
        stage_notice_files(project_root, release_dir)
        source_package, binary_package = create_release_packages(
            project_root,
            release_dir,
            metadata,
        )
        print(f"Release directory: {release_dir}", flush=True)
        print(f"Source package: {source_package}", flush=True)
        print(f"Binary package: {binary_package}", flush=True)
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
