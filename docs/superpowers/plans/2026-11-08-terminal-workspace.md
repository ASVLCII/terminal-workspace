# Terminal Workspace Implementation Plan

Goal: implement the master plan approved in chat on 2026-11-08. This plan and docs/design/DESIGN.md travel with the code. The written design controls appearance and behavior where illustrative screenshots differ.

Architecture: Windows x64, Rust 1.95, eframe/egui + wgpu, local egui_term adaptation over Alacritty, egui_dock without detached windows, local authenticated bridge to a bundled TypeScript OpenCode plugin. App-owned OpenCode backend is lazy and shared only among this app's terminals.

## Global constraints
- One native app window; independent real terminal processes; no browser renderer.
- Saved layouts, folders, launch definitions, settings and chat references; never automatically execute restored commands.
- Local application-data state, atomic replacement and last-known-good recovery. No model secrets or terminal transcripts saved by the app.
- Process ownership via Windows job objects. Never stop unrelated processes or a user's existing OpenCode service.
- Process-local plugin configuration. Do not modify global OpenCode configuration.
- Exact terminal/chat attachment targeting, no folder broadcasts, no automatic prompt submission.
- App-only private-byte budgets: 200 MiB empty; 350 MiB with four PowerShell panes/default scrollback and one 1920x1080 image. CLI/backend and GPU memory reported separately. No forced working-set trimming.
- Defaults 14 pt terminal text and 10,000 scrollback lines. Windows per-user NSIS installer, no automatic startup, preserve data on upgrades/uninstall by default.
- Out: IDE/editor, file explorer, Git dashboard, Antigravity, remote terminals, universal chat integration, command reruns, themes, auto updates.
- Each milestone has verified local evidence and a local commit. Publishing is not authorized.

## Review focus
- Same-folder concurrent OpenCode terminals cannot receive each other's attachments.
- TUI key/input, Vietnamese composition and terminal resize while output streams.
- Interrupted drags and maximize/restore never recreate processes or lose layout.
- Remembered background policy does not override explicit Quit.
- Save failures, live descendants and image/clipboard resources cannot be silently lost.

## Milestone 1: foundation gate
- [ ] Vendor the inspected terminal component with license/provenance; adapt to one egui version.
- [ ] Test isolated attachment batches first; implement authenticated bridge and bundled TUI/server plugin.
- [ ] Create one window with two actual terminals; verify PowerShell/OpenCode rendering, input, selection, clipboard, scrolling, Unicode, resizing and event-driven repaint.
- [ ] Prove same-folder targeting and staging of a file, folder and screenshot without submitting. Prove image reference to native preview.
- [ ] Stop before remaining screens if supported APIs cannot satisfy targeting/attachment requirements. Record exact blocker, do not silently reduce scope or change architecture.

## Milestone 2: workspaces and launch
- [ ] Sidebar, first-run state, footer, new-workspace dialog, terminal menu/custom command.
- [ ] Windows executable/ps1/cmd resolution, correct working folder, Unicode/spaces, meaningful errors.
- [ ] Real Starting/Running/Exited/Failed/Stopping states; no invented agent status.

## Milestone 3: panes
- [ ] Stable terminal identities; dock/resize/tab/drop previews, cancel/interruption restore.
- [ ] Columns/rows/2x2 presets without spawning processes; overflow retained as tabs.
- [ ] Maximize/restore exact layout; completed-operation persistence; native resize cursors/shared straight dividers.

## Milestone 4: images/attachments
- [ ] File/folder native pickers, drop target, screenshot paste, explicit recipient and supported semantics.
- [ ] Fit/zoom/pan/move/resize/close image panes, correct relative paths, bounded decoding and resource release.
- [ ] Retain clipboard files while referenced; local failure/recovery; preserve unsent input.

## Milestone 5: history
- [ ] Recent and paged OpenCode metadata/title search; resume by original identifier.
- [ ] Focus already-open chat, respect source project, recover missing refs. No transcript import/indexer.

## Milestone 6: settings/close/tray
- [ ] Font size, scrollback, shortcut reference, resettable closing policy.
- [ ] Ask/stop/background/cancel state machine; remember only committed checked action.
- [ ] Tray/reopen/single instance, usable-return-path guard; explicit Quit always exits after running-process warning.
- [ ] Save failure recovery; graceful stop then explicit force-stop for remaining owned children.

## Milestone 7: installer/release
- [ ] Per-user NSIS installer/uninstaller, shortcut, bundled plugin/assets/licenses; no bundled agent CLIs or global changes.
- [ ] Upgrade preserves state, uninstall retains user data by default; unsigned status explicit.
- [ ] Build/usage documentation and real Windows compatibility report.

## Acceptance
- Run focused tests red then green and full suite per meaningful unit. Build/check/fmt/clippy before release.
- Desktop checks: PowerShell/OpenCode/Codex; Claude only when installed; IME/Unicode/mouse/keys/clipboard/native dialogs/scaling/focus/high contrast/tray.
- Resize streaming output, process-preserving moves, canceled drags, duplicate prevention, all close policies, failed tray, corrupt state, missing folders/chats/images/commands/bridge.
- Measure warmed release host private bytes for empty and four-shell+image workloads; record children/GPU separately, repeated open-close and sustained-output retention.
- Compare actual app to all four boards, including the user's specific footer/button/keycap/divider corrections.
- No milestone or release is complete without evidence. A development build is not the finished app.
