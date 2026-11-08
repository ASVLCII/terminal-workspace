# Terminal workspace: interface design

Date: 2026-11-08. Status: design proposal for review, not an implementation or a performance result. “Terminal workspace” is a working title, not a chosen product name.

## Intent and authority

A small Windows desktop terminal host for ASVLCII. One window switches saved workspaces. Each workspace contains multiple real terminals. The actual CLI owns its terminal interface; the app supplies workspace navigation, pane management, previews, and supported attachment/history integrations.

Primary visual references: [Orca](https://github.com/stablyai/orca) and [SpaceVibe Deck](https://github.com/mxrsv/spacevibe-deck). Secondary references: [OpenCode](https://github.com/anomalyco/opencode) and [Xelif](https://github.com/ASVLCII/Xelif). Apply the user's design-taste foundation. Refinement guidance: [Unslop by Damir Sokolovsky](https://github.com/heydamir/unslop), CC BY 4.0, and [Impeccable](https://github.com/pbakaus/impeccable).

The written behavior below is authoritative where generated images simplify, omit, or misdraw a detail. Terminal transcripts, folders, times, counts, and images on the boards are illustrative. A generated screenshot cannot validate exact pixels, accessibility, interactions, terminal compatibility, or memory usage.

## Visual boards and coverage

| Board | Requested items covered |
| --- | --- |
| [01 Workspace and layouts](01-workspace-and-layouts-v2.png) | 1 sidebar, 2 terminals, 3 preview, 5 bottom bar, 8 pane layout, 15 process status |
| [02 Create, launch, resume, attach](02-create-launch-resume-attach.png) | 4 past chats, 6 new workspace, 7 new terminal, 9 attachments |
| [03 Settings and closing](03-settings-and-closing-v2.png) | 10 settings, 11 tray, 12 close warning, 13 quit warning |
| [04 Recovery and status](04-recovery-and-status.png) | 14 errors/recovery, 15 process status |

## Shared visual and interaction rules

- Quiet graphite surfaces, low saturation, readable light text; color indicates focus, links, or meaningful state. No ornamental gradients, texture, glow, or card grid.
- Proposed starting tokens: terminal background #151516, sidebar #1D1D1F, chrome #202022, primary text #EDEDED, secondary text #AAAAB0. Actual contrast must be measured in implementation.
- One understated Windows sans-serif for controls and one user-readable monospace for terminals. UI text 13–14 logical pixels; terminal size separately adjustable. Small controls approximately 32 logical pixels high. Spacing follows a 4-unit scale with small optical corrections.
- Sentence-case headings. Consistent neutral 16-pixel line icons. Modest 4–6-pixel rounding for controls; panes use single thin separators rather than nested cards.
- Sidebar starts around 256 logical pixels wide and can collapse. Pane headers and bottom bar align. Settings is inside the bottom-left footer, on the same baseline as current folder and terminal count.
- Only confirmation and small creation tasks use modal dialogs. Settings and full history use central app tabs; they do not create separate OS windows or stop terminals.
- Keyboard focus remains visible. Menus dismiss with Escape and return focus to their trigger. Modal focus stays inside the modal, then returns to the invoking control. Cancel is the initial focus for destructive confirmation.
- Long workspace/chat titles truncate with full text available on focus/hover. Error text and paths wrap. Status is conveyed by text and icon, not color alone. Vietnamese composition and Unicode paths are required implementation checks.
- On smaller windows the sidebar can collapse and the image pane can close or become a tab. Preserve terminal font size; do not shrink text to squeeze every pane in. Stop splitting when a readable minimum size cannot be maintained.
- Reduced-motion support is required. Drag overlays and focus feedback must not move terminal reading content unnecessarily. Timing remains unvalidated by these still images.

## 1. Workspace sidebar

Hierarchy: project folder → saved workspace → terminals. Multiple workspaces may share a project folder. Selecting a workspace restores its visible pane arrangement; selecting a terminal focuses its existing pane rather than launching a duplicate.

Each terminal row shows a useful name, tool or shell, and process status. Workspaces keep layouts, terminal working directories, launch definitions, and supported chat references. Do not label all saved terminal definitions Running after reopening; launch or resume requires an explicit action.

Past chats is a distinct section beneath current workspaces. Search scopes workspace/chat names and available history metadata; it is not a new full-text transcript indexing service.

First run: preserve the sidebar and footer; central empty state says “Create a workspace” with New workspace. Creating a workspace is also the recovery path when the saved list is empty.

## 2. Terminal area

Every terminal owns an independent real process. OpenCode, Claude Code, Codex CLI, PowerShell, and arbitrary commands retain their own terminal UI. App controls must not replace the CLI prompt or fabricate a second chat composer.

Compact pane header: tool/name, process state, maximize/restore control, close. Extra pane commands live in a context menu. Text selection, copy/paste, scrollback, Unicode, modifier keys, resize, and input methods are core correctness requirements.

Inactive workspaces may keep their processes running. Hiding a terminal is not stopping it. Closing a running terminal requires a scoped confirmation that names the affected terminal; stopping it must not silently terminate other terminals.

Exited output remains visible within the current bounded buffer. No automatic restart and no promise of persisted scrollback across app restarts. New terminal remains the explicit relaunch path.

## 3. Image preview pane

Click a recognized image reference to open a preview beside its source terminal. The source terminal keeps its scroll position. Resolve relative references against the correct source working directory. Do not steal ordinary clicks that a terminal application consumes; verified tool integration must establish clickable references.

Header: filename, drag handle, close. Controls: Fit, zoom out/in, current zoom percentage; dragging a zoomed image pans it. Resetting to Fit recenters it. Resizing in Fit recalculates the fit.

This pane uses the same move/resize/dock system as terminals. Closing a preview releases its decoded image resources. Decode only when needed and constrain very large images; no memory savings have yet been measured. Missing/unsupported/unreadable images use the recovery view on board 04.

## 4. Past chats

Sidebar shows a short recent list; View all opens a central Past chats tab. Controls: project scope, tool scope (OpenCode initially), search by available metadata. List columns: chat title and updated time. Selected-row detail shows title, tool, source folder, and Resume in new terminal.

Resume targets the original chat identifier and source project when supported. If it is already open, focus its existing terminal rather than launch a duplicate. If the current workspace folder differs, ask whether to open the source workspace; do not silently resume in an unrelated folder.

No automatic transcript import or universal cross-tool history. Empty history says “No OpenCode chats found” with Open new chat. An unavailable bridge gives a local explanation and Try connection, while ordinary terminals continue working.

## 5. Bottom bar

Left: Settings inside the sidebar footer. Main segment: focused terminal's working folder, or workspace folder when a nonterminal tab is focused. Right: total terminal panes in the selected workspace, including exited panes; running count may be clarified in a tooltip. Preview panes do not count as terminals.

No unmeasured RAM number, model quota, or activity dashboard. Folder text can truncate; full path is available for copy.

## 6. New workspace

Small modal with Name, Project folder + Browse, and Start with. Use the native Windows folder picker. Proposed default name derives from the folder, editable; proposed initial shell is PowerShell.

Start with uses the same terminal choices as New terminal. Selecting Custom command reveals Command; it does not send a command while merely selecting an option. Cancel preserves the existing workspace. Create workspace validates the folder and command selection, saves the workspace, then launches exactly one initial terminal. Guard against repeated clicks creating duplicate terminals.

Missing/unreadable folders show an inline field error and preserve entered values. No Git initialization, worktree creation, account setup, or install workflow is added.

## 7. New terminal menu

Anchor to + Terminal. Choices: PowerShell, OpenCode, Claude Code, Codex CLI; separator; Custom command…. Quiet footer displays the target folder. Opening a predefined tool starts one terminal in the selected workspace directory, using the user's existing tool configuration without adding permission-bypass flags.

Custom command opens a compact form with Command and Working folder + Browse. Open terminal is explicit execution. An unavailable command offers Edit command or Open PowerShell. Do not auto-install software.

## 8. Pane layout controls

Layout-button popover: Columns, Rows, 2 × 2 grid, then Maximize pane / Restore layout where applicable. Simple schematic line icons illustrate each arrangement. Apply presets to existing panes; never launch additional processes to fill slots. Unfilled cells remain empty drop locations. More than four panes remain reachable as tabs; do not hide or stop them to force a 2 × 2 layout.

Drag headers/tabs toward a pane edge to preview left, right, above, or below; drop centrally to tab-group with that pane. One blue translucent target shows the result before release. Escape, lost pointer capture, or cancel restores the prior layout without moving or stopping processes. No new OS window is created by dragging out.

Dividers have a larger invisible hit area and matching resize cursor. Double-clicking a divider equalizes the adjacent split. Keyboard-accessible pane commands offer the same arrangements. Maximize temporarily hides siblings without terminating them; Restore returns exact prior proportions. Save the layout after a completed operation, not on every pointer movement.

Grid panes meet at straight shared dividers without card gutters. A horizontal divider uses the normal vertical resize cursor; a vertical divider uses the horizontal resize cursor. Highlight only the divider being hovered or dragged. Do not add a large arrow/handle at the central crossing. Show Restore only while a pane is actually maximized.

## 9. Attachments

Compact paperclip menu: Add files…, Add folder…, Paste image. Native Windows pickers handle selection. Show the target terminal explicitly. Drag-over highlights only the prospective target pane; dropping commits to that target, not whichever pane was active before the drag.

Insert through a verified tool-specific integration. Never send Enter or submit automatically. Folder attachment is a directory reference, not an unbounded recursive read. Preserve typed input on failure. A generic shell can receive a correctly quoted path reference, but must not call that a successful agent attachment.

Screenshot paste receives a readable filename and an app-managed local file accessible to the target tool. Do not delete a file while an active prompt/chat still depends on it; cleanup ownership must be implemented before promising automatic removal.

The board's filename tokens represent the integration's staged references, not an independent app-owned chat composer. Removing a staged token removes that reference from the unsent prompt only. Integration failure offers Copy path and Try connection and does not silently substitute a different action.

## 10. Settings

Open one Settings tab and reuse it. Three flat sections: Terminal, Closing behavior, Keyboard shortcuts. Changes apply immediately, with invalid values rejected inline; no unnecessary Save dialog.

Terminal: font-size stepper and preview, plus bounded scrollback. Proposed initial values are 14 pt and 10,000 lines per terminal; these are design starting points, not validated memory limits. Explain that older lines are discarded. Do not expose Unlimited by default.

Closing behavior values: Ask every time (initial), Stop terminals and quit, Run in background. Setting Ask every time resets the saved choice from the warning. Background helper explains that processes continue running and using memory.

Keyboard shortcuts: a small reference list, not a full keybinding editor. Proposed chords in the board are provisional; terminal input compatibility takes precedence. Conflicting chords must either be changed or apply only when app chrome is focused. Do not advertise a chord as both app navigation and transparently passed to a TUI in the same focus context.

Shortcut keycaps must sit vertically centered inside consistent rows, with at least 8 logical pixels between their outer borders and adjacent divider lines. A 24-pixel keycap therefore requires a row at least 40 pixels high. Keep the helper below the list separate from the final row.

## 11. System tray

Background mode hides the main window and exposes a tray icon. Menu: Open window; noninteractive count of running terminals; Quit. Activating the icon or relaunching the app reopens the same window/process instead of starting duplicate terminal hosts.

Check that a usable tray return path exists before hiding. If it cannot be created, keep the window visible and show a local explanation. No separate resident service is part of this design.

## 12. Close warning and remembered preference

Default window-close action opens Close window?. Show the running-terminal count, a compact list (scroll if long), and “Stopping them may interrupt work.” Actions: Cancel, Run in background, Stop and close. With no running terminals, adapt the copy to “Save this workspace and close?” rather than invent a running count.

Checkbox: “Don't show this again”. Helper: “Remember the action you choose below.” It is initially unchecked. Checking alone does nothing; store a preference only when the user selects one of the two committing actions.

| Checkbox/action | Result | Stored preference |
| --- | --- | --- |
| Unchecked + Stop and close | Save state, stop terminals, exit | Unchanged |
| Checked + Stop and close | Save state, stop terminals, exit | Stop terminals and quit |
| Unchecked + Run in background | Save state, hide to tray, keep terminals | Unchanged |
| Checked + Run in background | Save state, hide to tray, keep terminals | Run in background |
| Either + Cancel, Escape, or dialog X | Keep window and terminals open | Unchanged |

Future window-close events follow a stored action without repeating this warning. Cancel must not be saved as a closing policy. A stop operation should first request normal process shutdown; if processes remain, show their names and an explicit Force stop / Cancel choice rather than claim successful exit or silently force-stop.

## 13. Explicit quit warning

Tray Quit always means exit, never hide again. With running terminals, title “Quit Terminal workspace?”; body “Stop N running terminals and exit the app?”; helper “Workspace layouts and chat references will be saved.” Actions: Cancel and Stop terminals and quit. No background choice and no remember checkbox. With no running terminals, save and exit directly.

Remembered window-close policy cannot override explicit Quit. Save failures must be reported before discarding recoverable state; preserve the window and allow retry. Stopped agent work is not guaranteed saved merely because layout/chat references were saved.

The quit dialog's two-button group is horizontally centered as a whole, with equal outside margins and a consistent 16-pixel gap. Button widths follow their labels; centering individual labels alone does not balance the action group.

## 14. Error and recovery patterns

Use a local pane/workspace error rather than blocking the whole app. Keep useful context and typed input. Explain the problem and one or two useful next actions; technical details can expand on demand.

| Condition | Message/action |
| --- | --- |
| Project folder missing | Project folder not found; Locate folder / Try again. Remove workspace… removes saved metadata only, after confirmation. |
| Command absent or cannot launch | Failed to start with command/folder; Edit command / Open PowerShell; expandable Details. |
| Chat reference unavailable | This chat could not be reopened; Try again / Open new chat. Do not delete the history reference automatically. |
| Image missing or unreadable | Image unavailable; Locate image / Try again. Close still works. |
| Attachment integration unavailable | Identify affected terminal; Copy path / Try connection. Preserve the unsent prompt. |
| No chat search matches | No matching chats; Clear search. Do not imply history was deleted. |

Retry acts only on the failed operation and prevents concurrent duplicate requests. Do not silently rerun a submitted prompt as part of recovery.

## 15. Process status

Use process facts: Starting, Running, Exited (code 0 when known), Failed (nonzero code or launch failure), Stopping. Unknown exits say Exited, not Failed. An existing shell stays Running even when a foreground command inside that shell returns a nonzero status; only label foreground status separately if reliable integration supplies it.

Running does not mean thinking, idle, or ready. Agent-specific activity is outside the initial baseline. Show the same state vocabulary in sidebar, pane header, close warning, and tray count. Settings/history/preview tabs never display terminal process dots.

## Scope and implementation checks

Native Rust host remains the preferred architecture. OpenCode's optional plugin/bridge may require the tool's supported plugin language; “native Rust app” must not be misrepresented as “all hosted programs and integrations are Rust.” No full IDE, editor, file explorer, Git dashboard, Antigravity integration, automated command reruns, or general transcript indexing is added.

Before calling this implemented: verify real terminal keyboard/render/Unicode/resize behavior, layout save/restore and drag cancellation, CLI-specific attachments/history, paths containing spaces, close preference persistence, actual tray lifecycle, failure recovery, focus/contrast/high-contrast behavior, and measured host/child-process memory separately. Generated boards provide no evidence for those checks.

## Artifacts

Boards were produced with the built-in image generation tool. Prompts specified the accepted Orca/SpaceVibe identity, the exact surfaces and labels, and preserved scope; board 03 received a targeted correction for footer placement, process wording, and illustrative date. Original generated files are retained. This handoff contains static raster concepts and a written interaction specification, not source UI or motion validation.
