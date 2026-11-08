# Implementation record

## Milestone 1: terminal/integration foundation

Implemented native eframe/wgpu window with two independently owned ConPTY terminals, a vendored egui_term adapter on egui 0.36.2, a local authenticated attachment bridge, and an app-owned OpenCode backend. Bundled plugin loads through per-process configuration; no global plugin configuration is changed.

Evidence obtained on Windows 11 x64 with OpenCode 2.2.5:
- Rust workspace suite: 12 unit tests passed (8 terminal component, 4 app/bridge).
- Real Rust-host → authenticated bridge → OpenCode plugin admission test passed. A file, a folder reference and an image enter chat A; chat B in the same directory remains empty. Admission uses resume:false, never requests model execution, and acknowledges staged files after durable enqueue.
- Actual native window with two OpenCode TUIs: both handshakes succeed, A has three staged references, B has none. No prompt is submitted by that desktop check.
- Desktop input regression: typing reaches the focused shell even with the pointer outside it. Observed failing before the fix and passing after.
- Desktop image-link regression: a normal click on a file image URI opens the native decoded preview. Observed failing before hover/locking corrections and passing after.
- Clipboard byte tests: negotiated bracketed paste and UTF-8 text preserve their content without substituting Ctrl+V.

Rulings and fixes:
- Fresh repo on implement/native-host provides isolation; no existing checkout was changed.
- Cargo cache/build output are project-local on Z: because C: has no free space. Use scripts/build.ps1 to locate this machine's existing portable MSVC/SDK; normal developer shells need no fallback.
- The host talks only to local HTTP services, so reqwest has no unnecessary TLS provider or C build dependency.
- Native staging controls display references; the TUI plugin uses host APIs and a status toast instead of JSX/runtime rendering dependencies. This avoids OpenCode's Node loader resolving JSX through React and requiring separate Solid packages.
- Terminal rendering copies visible cells only; it no longer clones the entire scrollback grid per repaint.

Remaining first-release checks include physical Vietnamese IME composition, the complete keyboard/mouse matrix, scaling/high contrast, process shutdown failure paths, and release memory workloads. The foundation build is not a finished release.

## Remaining milestones

2. Workspaces, persistence and launching: pending.
3. Docking, saved layouts and maximize: pending.
4. Complete image/attachment controls: pending.
5. OpenCode history/resume: pending.
6. Settings, close state machine and tray: pending.
7. Installer, full acceptance and release memory measurements: pending.
