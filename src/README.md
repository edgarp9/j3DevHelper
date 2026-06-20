# j3DevHelper

Workspace-oriented command helper for desktop use.

## Linux Distribution

Bundle these files together for Linux desktop use:

- `j3devhelper` executable
- `icon.svg`
- `icon.png` fallback, recommended for environments that cannot load the SVG

Desktop integration is explicit and user-scoped:

```bash
./j3devhelper --install
./j3devhelper
./j3devhelper --uninstall
```

`--install` writes the desktop entry and hicolor icon files under the XDG user data directory. The Linux application id is `io.github.edgarp9.j3DevHelper`; because it contains uppercase characters, install also writes the lowercase Plasma fallback alias `io.github.edgarp9.j3devhelper` with `NoDisplay=true`.
