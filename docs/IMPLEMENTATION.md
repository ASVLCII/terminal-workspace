# Implementation and acceptance record

Date: 2026-11-08. Working title: Terminal Workspace. Windows x64 development build; no publication or signing is claimed.

## Implemented

| Milestone | Local implementation |
| --- | --- |
| 1. Foundations | Native eframe/wgpu host, app-owned ConPTY terminals, local authenticated per-run bridge, bundled OpenCode server/TUI plugin, exact chat targeting, native image URI preview |
| 2. Workspaces | Sidebar, empty state, asynchronous folder validation/pickers, saved definitions, PowerShell/OpenCode/Claude/Codex/custom launching, status and recovery errors |
| 3. Panes | Resizable docking, directional/center drops, tab overflow, presets, temporary maximize, stable process identities, saved proportions, Escape drag restoration |
| 4. Media | File/folder references, drag/drop recipient highlighting, retained clipboard PNGs, Fit/zoom/pan, bounded background decoding and texture release |
| 5. History | Project-scoped OpenCode title search and bounded metadata pages, exact chat resume/focus, source-project confirmation |
| 6. Lifecycle | Font/scrollback settings, shortcut reference, remembered close choice, tray, single-instance activation, save acknowledgements, normal/force shutdown and scoped terminal stopping |
| 7. Packaging | Release executable, per-user NSIS recipe, bundled plugin/license inventory, Start-menu/uninstall handling, unsigned-build labeling. Acceptance work continues below. |

Implementation status does not imply every release gate has passed.

## Verified so far

- Rust workspace: 33 unit tests passed (9 terminal component, 24 host/model/bridge/recovery/process tests). The live OpenCode test is opt-in and passed separately.
- Installed OpenCode 2.2.5: two chats in one directory, file/folder/image admission only to the selected chat, no automatic prompt, durable acknowledgement, untouched other chat, metadata listing and source project. Test admissions use `resume:false`; no model execution is requested.
- Real native window: two OpenCode TUI handshakes with three staged references in A and none in B; image-link click opens a native preview.
- PowerShell: real keyboard command execution, input with pointer outside the pane, native workspace creation/save, restored definitions remain stopped, layout operations preserve the same shell PIDs.
- Windows clipboard: screenshot paste creates a retained app-owned PNG; normal Unicode text/bracketed paste has byte-level tests.
- Closing: remembered Background hides subsequent closes; a second executable launch reopens the same instance; real tray Quit warns despite the remembered background choice and stops the owned shell on confirmation.
- Ownership: fast descendants inherit the terminal job before any child instructions execute; stopping an owned job leaves an independently launched process alive.
- Recovery: corrupt primary JSON falls back to a good backup; future versions are preserved; invalid proportions and duplicate terminal identities are rejected.
- Focus regression: a real history search previously entered a neighboring terminal. The reproducer failed on the older release binary and passed with the corrected widget focus routing.
- Vietnamese combining-mark regression: decomposed marks are retained with the base glyph (failed before the renderer correction, passed after). Physical IME acceptance is still separate.

- Default terminal font: installed CaskaydiaMono NFM, including its localized family alias. Native font-picker round trip through Consolas and back saved correctly and retained all existing terminal process IDs. Image wheel zoom changed 18% to 27% in the real preview.
- UI revised against the four boards: grouped sidebar and pinned footer, compact pane headers, full-main Settings/Past Chats, native line icons, flat surfaces and aligned controls. Final multi-scaling visual acceptance remains open.
- Quitting during delayed OpenCode startup: older release lost the chat reference (regression failed); corrected debug preserved it. Starting-terminal removal is guarded before layout/state mutation. Independent review identified descendant relaunch and final-save ordering edges; fixes preserve live jobs and acknowledge a final save after shutdown.

## Memory investigation

Private bytes, release build, this PC (Intel Iris Xe), without trimming:

- Initial default backend: empty window peak 281.78 MiB (fails 200 MiB), four shells + 1920×1080 RGB PNG peak 316.10 MiB (passes 350 MiB).
- Controlled backend comparison using the same executable: DirectX 12 empty window 117.32 MiB; Vulkan 255.57 MiB. These are diagnostic short samples, not the final workload acceptance numbers.
- Current release executable SHA-256: `10b19aebab831c343aa95aa698a1befd17aef77aa82f0f1ff10fd57694c4e998`. DirectX 12, CaskaydiaMono NFM 14 pt, 10,000 lines, 12-second warm-up and 12 samples: empty peak **162.30 MiB**, four PowerShell panes plus the 1920×1080 image peak **204.82 MiB**. Both app-only budgets pass.
- Separate final-sample four-shell figures: hosted processes **242.90 MiB** private bytes, total process tree **447.73 MiB**; GPU **97.86 MiB shared**, **0 dedicated**. Empty GPU allocation: **93.79 MiB shared**. GPU counters are allocation observations, not private-byte budget inputs. No working-set trimming. Retained-growth and representative OpenCode workload gates remain below.

Detailed local test artifacts are under artifacts/ (ignored to avoid committing generated transcripts/media payloads). scripts/benchmark.py creates isolated data and records private-byte workloads; --benchmark-workload is an explicit test mode and never runs on ordinary reopening.

## Remaining release gates

- Retained-growth/output cycles and representative OpenCode application-tree measurement. The two specified release RAM workloads and separate GPU counter capture passed.
- Complete third-party notice audit before external distribution. Current unsigned NSIS installer passed install/upgrade/uninstall: payload and Start-menu shortcut removed, application data and unknown added files retained.
- Full keyboard/mouse reporting/modifier matrix, physical Vietnamese IME composition, Unicode/space paths, keyboard-only flows, Windows scaling/high contrast and long-label comparisons against all four boards.
- Codex interactive compatibility check; Claude Code on a machine where it is installed. Claude is not installed here and must not be silently installed by this app.
- Deliberate no-tray/save/shutdown failure paths and interrupted pane dragging. Independent source review completed with its reported lifecycle/persistence issues corrected and re-reviewed; native delayed-startup quit regression passed on both debug and the packaged release source build.

## Architecture rulings

- This new standalone repository and implement/native-host branch provide isolation. No existing checkout was changed; no Git remote or publishing is configured.
- Build caches remain on Z: because C: is full. The approved LocalAppData junction points only this app’s data to Z:.
- OpenCode’s supported TUI host APIs provide association/status without JSX/Solid dependencies. Staged references appear in native controls; the real CLI retains its prompt.
- Ctrl+Shift+I pastes a screenshot because egui-winit consumes Ctrl+Shift+V as text paste before application key handlers.
- A narrow Apache-licensed Alacritty Windows adaptation assigns ConPTY children to their job while suspended, before resuming them, and closes the primary-thread handle. The stdio backend uses the same suspend/assign/resume ordering.
- Only visible terminal cells are copied for rendering; complete history grids are never cloned per frame. Repaint wakeups are coalesced by egui rather than queued through the application event channel.
- DirectX 12 is the Windows default because measured Vulkan fixed overhead misses the agreed idle budget on this PC. WGPU_BACKEND remains a diagnostic override.

## Current local artifacts

- `dist/app/terminal-workspace.exe`: standalone development application with adjacent bundled plugin.
- `dist/TerminalWorkspace-0.1.0-dev-x64-setup.exe`: unsigned per-user development installer; SHA-256 `7a078ae837c86aa0c90f30c9a429265eec9b81d04104d93a08e4e3d121e7caee`.
- Repeatable checks: `scripts/design-verification.py`, `scripts/desktop-font-zoom.py`, `scripts/desktop-quit-startup.py release`, `scripts/benchmark.py`, `scripts/installer-smoke.py`. The desktop checks run sequentially in an interactive Windows session; each uses isolated app-owned data/processes.
- Clippy completed with advisory style warnings; it is not reported warning-free.
