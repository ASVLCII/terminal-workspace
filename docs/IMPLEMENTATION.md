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

- Rust workspace: 29 unit tests passed (9 terminal component, 20 host/model/bridge/recovery/process tests). The live OpenCode test is opt-in and passed separately.
- Installed OpenCode 2.2.5: two chats in one directory, file/folder/image admission only to the selected chat, no automatic prompt, durable acknowledgement, untouched other chat, metadata listing and source project. Test admissions use `resume:false`; no model execution is requested.
- Real native window: two OpenCode TUI handshakes with three staged references in A and none in B; image-link click opens a native preview.
- PowerShell: real keyboard command execution, input with pointer outside the pane, native workspace creation/save, restored definitions remain stopped, layout operations preserve the same shell PIDs.
- Windows clipboard: screenshot paste creates a retained app-owned PNG; normal Unicode text/bracketed paste has byte-level tests.
- Closing: remembered Background hides subsequent closes; a second executable launch reopens the same instance; real tray Quit warns despite the remembered background choice and stops the owned shell on confirmation.
- Ownership: fast descendants inherit the terminal job before any child instructions execute; stopping an owned job leaves an independently launched process alive.
- Recovery: corrupt primary JSON falls back to a good backup; future versions are preserved; invalid proportions and duplicate terminal identities are rejected.
- Focus regression: a real history search previously entered a neighboring terminal. The reproducer failed on the older release binary and passed with the corrected widget focus routing.
- Vietnamese combining-mark regression: decomposed marks are retained with the base glyph (failed before the renderer correction, passed after). Physical IME acceptance is still separate.

## Memory investigation

Private bytes, release build, this PC (Intel Iris Xe), without trimming:

- Initial default backend: empty window peak 281.78 MiB (fails 200 MiB), four shells + 1920×1080 RGB PNG peak 316.10 MiB (passes 350 MiB).
- Controlled backend comparison using the same executable: DirectX 12 empty window 117.32 MiB; Vulkan 255.57 MiB. These are diagnostic short samples, not the final workload acceptance numbers.
- Windows now defaults to wgpu DirectX 12 and memory-oriented allocation. Full workloads must be repeated against the final build, including the installed system fonts. Hosted CLI/backend and GPU memory are separate metrics.

Detailed local test artifacts are under artifacts/ (ignored to avoid committing generated transcripts/media payloads). scripts/benchmark.py creates isolated data and records private-byte workloads; --benchmark-workload is an explicit test mode and never runs on ordinary reopening.

## Remaining release gates

- Final release workloads and retained-growth/output cycles; GPU accounting and representative OpenCode application-tree measurement.
- Final installer install/upgrade/uninstall preservation test and complete third-party notice audit before external distribution.
- Full keyboard/mouse reporting/modifier matrix, physical Vietnamese IME composition, Unicode/space paths, keyboard-only flows, Windows scaling/high contrast and long-label comparisons against all four boards.
- Codex interactive compatibility check; Claude Code on a machine where it is installed. Claude is not installed here and must not be silently installed by this app.
- Deliberate no-tray/save/shutdown failure paths, interrupted pane dragging, and final independent code review.

## Architecture rulings

- This new standalone repository and implement/native-host branch provide isolation. No existing checkout was changed; no Git remote or publishing is configured.
- Build caches remain on Z: because C: is full. The approved LocalAppData junction points only this app’s data to Z:.
- OpenCode’s supported TUI host APIs provide association/status without JSX/Solid dependencies. Staged references appear in native controls; the real CLI retains its prompt.
- Ctrl+Shift+I pastes a screenshot because egui-winit consumes Ctrl+Shift+V as text paste before application key handlers.
- A narrow Apache-licensed Alacritty Windows adaptation assigns ConPTY children to their job while suspended, before resuming them, and closes the primary-thread handle. The stdio backend uses the same suspend/assign/resume ordering.
- Only visible terminal cells are copied for rendering; complete history grids are never cloned per frame. Repaint wakeups are coalesced by egui rather than queued through the application event channel.
- DirectX 12 is the Windows default because measured Vulkan fixed overhead misses the agreed idle budget on this PC. WGPU_BACKEND remains a diagnostic override.
