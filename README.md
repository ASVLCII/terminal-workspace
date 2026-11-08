# Terminal Workspace

Windows-first native Rust terminal workspace. Development implementation of the approved [master plan](docs/superpowers/plans/2026-11-08-terminal-workspace.md) and [interface design](docs/design/DESIGN.md).

Current status and verification: [implementation record](docs/IMPLEMENTATION.md). This is not the finished release.

## Build

Rust 1.95 or newer, Windows x64 MSVC build tools and a Windows SDK are required.

```powershell
./scripts/build.ps1 -Action build
./scripts/build.ps1 -Action test -Workspace
```

The helper keeps build artifacts off this machine's full C: drive. In a configured developer shell, ordinary cargo commands also work when CARGO_HOME points at a volume with free space.

## Foundation checks

```powershell
node --test plugin/test/core.test.mjs
python scripts/desktop-foundation.py
python scripts/desktop-opencode.py
python scripts/desktop-image.py
```

The desktop scripts require pywinauto and an interactive Windows desktop. They launch and close only their own test app. The OpenCode gate uses the installed CLI and local plugin, without submitting a model request.

The live Rust integration check is opt-in: `cargo test --test opencode_live -- --ignored` in the configured build environment. It creates and removes only its own test sessions.

Original egui_term author/license and local adaptation provenance are retained under vendor/egui_term.
