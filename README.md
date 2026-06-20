# j3DevHelper

Workspace-oriented command helper for desktop development.

j3DevHelper is a small in-house desktop tool for managing folder-based workspaces and running reusable command buttons against the selected workspace. It is designed for developers who frequently switch between projects and want a simple native UI for opening tools, running Git commands, launching build commands, and organizing project-specific workflows.

## GitHub Description

In-house, AI-assisted desktop command helper for managing workspace-specific command groups on Windows and Linux.

## Status

This project was built as an in-house tool with AI-assisted development. It is useful for the author's workflow, but it should still be treated as early-stage software.

Automated tests exist, but test coverage is not sufficient yet. In particular, desktop UI behavior and platform-specific command execution need more manual and automated verification before this should be considered production-grade.

## Features

- Manage workspaces as folder-based entries with a display name and language.
- Organize workspaces with first-level categories.
- Create command groups and command buttons for repeated development tasks.
- Run commands directly through the OS shell API or in an external terminal.
- Use argument tokens such as `{path}`, `{name}`, `{Language}`, `{selectfile}`, `{selectdir}`, and `{inputtext}`.
- Save settings in a TOML file next to the executable, or use a custom settings file path.
- Switch UI language between English and Korean.
- Configure UI font, font size, theme, window size, and panel width.
- Use native Windows Win32 UI on Windows and GTK4 UI on Linux.

## Repository Layout

The Rust application lives in the `src/` directory.

```text
.
|-- LICENSE
|-- README.md
`-- src/
    |-- Cargo.toml
    |-- src/
    |-- docs/
    |-- icon.ico
    |-- icon.svg
    |-- j3devhelper.toml
    `-- j3devhelper-linux.toml
```

## Build

Install Rust, then run Cargo commands from the application directory:

```bash
cd src
cargo build
```

For a release build:

```bash
cd src
cargo build --release
```

The helper script can also build release artifacts and open the release directory:

```bash
cd src
python build_release.py
```

## Run

```bash
cd src
cargo run
```

You can pass a TOML settings file path as the only positional argument:

```bash
cargo run -- ./j3devhelper-linux.toml
```

If no settings path is provided, j3DevHelper uses a TOML file based on the executable name in the executable directory.

## Linux Desktop Integration

On Linux, bundle the executable with `icon.svg` and optionally `icon.png`.

```bash
./j3devhelper --install
./j3devhelper
./j3devhelper --uninstall
```

`--install` writes the desktop entry and hicolor icon files under the XDG user data directory. The Linux application id is `io.github.edgarp9.j3DevHelper`; because it contains uppercase characters, install also writes the lowercase Plasma fallback alias `io.github.edgarp9.j3devhelper` with `NoDisplay=true`.

## Notes

- This is not a polished commercial product. It reflects an in-house workflow and may need adjustment for other environments.
- Windows behavior should be manually verified on a real Windows host when changing Win32 UI or command execution behavior.
- Linux behavior depends on GTK4 and the available terminal emulator.
- Review command definitions carefully before running them. Command buttons can launch external programs and shell commands.

## Documentation

Additional implementation notes are available under `src/docs/`:

- `src/docs/domain.md`
- `src/docs/ui-spec.md`
- `src/docs/platform-equivalence.md`

## License and Notices

This project is licensed under the GNU General Public License v3.0. See `LICENSE` for details.

This project uses icons from [Google Fonts Icons](https://fonts.google.com/icons), also documented as [Material Symbols](https://developers.google.com/fonts/docs/material_symbols). Material Symbols are provided by Google under the [Apache License Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).

Thank you to Google and the Material Symbols / Google Fonts Icons contributors for making these icons available.
