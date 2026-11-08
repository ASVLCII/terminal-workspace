# Terminal Workspace

A Windows-first native Rust terminal host: one window, saved projects, independent terminals and adjustable panes. OpenCode adds staged attachments, image references and project chat history.

**Status: unsigned development build.** Core features are implemented and under acceptance testing. This is not yet a signed or fully validated first release. See [verification status](docs/IMPLEMENTATION.md) and the [master plan](docs/superpowers/plans/2026-11-08-terminal-workspace.md).

## Run

Build with Rust 1.95+ and the Windows x64 MSVC tools/SDK:

```powershell
./scripts/build.ps1 -Action build -Release
./target/release/terminal-workspace.exe
```

On this machine the helper uses the existing portable MSVC/SDK and keeps build caches on Z:. Standard cargo commands also work from an appropriately configured developer shell.

The packaged executable is `dist/app/terminal-workspace.exe`. The installer is `dist/TerminalWorkspace-0.1.0-dev-x64-setup.exe`. OpenCode, Claude Code and Codex CLI must be installed separately and available on PATH; they are never installed by this app.

## Daily use

1. Create a workspace and choose a project folder. Its first PowerShell starts explicitly as part of creation.
2. Use **+ Terminal** for PowerShell, OpenCode, Claude Code, Codex CLI, or a custom PowerShell command.
3. Drag tabs into highlighted zones to split or group them. Drag straight dividers to resize; double-click a divider to equalize. Columns, Rows and 2×2 reuse existing panes. Extra panes remain as tabs. Maximize keeps the same process; Escape restores the arrangement.
4. Use **Attach** in the recipient pane. Files, folder references and screenshots are staged for that OpenCode chat. Other terminals receive quoted path references. Nothing sends a prompt automatically. **Ctrl+Shift+I** pastes a screenshot; Ctrl+V remains text paste. Click a local image URI to preview it; registered image references also appear in the pane’s Images menu. Previews support Fit, zoom and drag-to-pan.
5. Open **Past chats** for current-project OpenCode metadata, title search and bounded pagination. Selecting a chat focuses an existing terminal or explicitly resumes it. Other tools’ chat history is outside this release.

Settings is pinned in the bottom-left footer. It controls font size, bounded scrollback and closing behavior, with a shortcut reference. Default terminal text is 14 logical points and scrollback is 10,000 lines.

## Saving, closing and background mode

State is versioned JSON under `%LOCALAPPDATA%\TerminalWorkspace`. Saves use atomic replacement and a last-good backup. No model credentials or terminal transcripts are saved in application state. OpenCode owns its own existing data/configuration. A bundled plugin is supplied only to app-owned processes.

Saved terminal definitions reopen **stopped**. Commands never rerun automatically. Closed-image pixels/textures are released; clipboard PNG files remain accessible because their references may still be in a chat.

Window closing asks **Cancel / Run in background / Stop and close**. Checking **Don’t show this again** remembers the committing action; Cancel/Escape/dismissal never save it. Settings can restore Ask every time. Background keeps terminals alive and hides the window. Reopening the executable activates the same instance. Tray **Quit** always means exit, regardless of the remembered window-close choice. Owned processes receive normal shutdown first; Force stop is a separate choice for those that remain.

On this PC, the user-approved LocalAppData junction points to `Z:\Codex\Stuff\terminal-workspace-data` because C: is full. The installer itself supports choosing a different program folder. Uninstall retains application data by default.

## Verification and packaging

```powershell
./scripts/build.ps1 -Action test -Workspace
node --test plugin/test/core.test.mjs
# In the helper-configured cargo environment:
cargo test --test opencode_live -- --ignored
./scripts/build.ps1 -Action build -Release
python scripts/benchmark.py
./scripts/package.ps1 -SkipBuild -MakeNsis 'path/to/makensis.exe'
```

Desktop scripts require an interactive Windows desktop, Python, pywinauto and psutil. They exercise their own test processes/data. Live OpenCode gates create only their own chats and submit admission with `resume:false`, without requesting model execution. Run desktop scripts sequentially because the app is single-instance.

The Windows renderer defaults to wgpu DirectX 12 with memory-oriented allocation. `WGPU_BACKEND=vulkan` is a troubleshooting override and has substantially higher measured fixed RAM on this PC. RAM checks use release private bytes, with hosted processes and GPU allocation reported separately; there is no working-set trimming.

The NSIS installer is per-user, creates a Start-menu shortcut and supplies uninstall support. It adds no startup task, service or updater. Builds are currently unsigned. Locked dependencies and available license/font notices are included in the payload. Microsoft system fonts are loaded from Windows and are not redistributed. Final application licensing and branding remain undecided; publishing is not authorized.
