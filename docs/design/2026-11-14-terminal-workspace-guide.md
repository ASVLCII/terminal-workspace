# Terminal Workspace design guide

Status: proposed direction, 14 November 2026. Baseline: `main` at `67e2590`. This guide is for review, not a claim that the app implements or passes every recommendation.

## Direction

A quiet, glass-backed terminal workspace: one clear navigation column, one dominant working area, and controls placed where their action happens. The video visually progresses from a crowded dashboard to clearer grouping, consolidated actions and frosted surfaces. Apply those principles to terminal work, not dashboard metrics. This guide is based on sampled video frames; the spoken audio was not transcribed.

The current build already introduces translucent surfaces, a custom title bar, accent selection and resizable panes. Its surface tokens lean blue/cyan and add edge sheen. The earlier preference was neutral black and gray; the newer glass commit may represent a change in taste. Use restrained neutral glass as the proposed default and preserve accents for purposeful states until the user decides otherwise.

## Priorities

1. Keep terminal text readable over both bright and dark desktop backgrounds. Use a sufficiently opaque neutral backing behind terminal output; glass belongs primarily in the window frame, sidebar and transient surfaces. Tune opacity through real tests, not a universal magic percentage.
2. Give running terminals and image previews most of the window. Show workspace hierarchy and process status without letting navigation dominate.
3. Group related actions by task. Put new terminal and pane layout in the workspace toolbar; put attach controls with the target terminal and preview controls with the image. Remove duplicated controls only after an accessible equivalent is confirmed.
4. Align labels, controls and dividers to shared edges. Let spacing establish groups before introducing another border or surface.
5. Make active, hovered, focused, disabled and error states distinguishable without relying on color alone. Keep keyboard focus visible against the actual backdrop.

## Surfaces and type

- Use a neutral near-black foundation, slightly lighter navigation and controls, and one-pixel separators. Avoid decorative gradients, repeated inner borders, strong shadows and persistent glints.
- Treat translucency as context, not the content: keep one consistent backdrop across adjacent chrome. Menus and warnings need a more solid surface so text remains legible against changing content.
- Use the existing Windows UI typeface for controls and the selected monospace font for terminal text. Keep the confirmed CaskaydiaMono NFM default where installed; respect the user's font selection and size.
- Use restrained type hierarchy: workspace and pane names first, status and helper text second. Avoid all-caps section labels unless they improve scanning in the actual screen.
- Use a consistent icon stroke and clear text labels for important actions. Color signals focus, selection or process status; status is also written in words.

## Main window and panes

- Sidebar: project → workspace → terminal, with compact rows, clear indentation, a subtle selected row and readable status. Pin Settings to the bottom left, aligned with the bottom bar. Allow long names to truncate with a way to see the full name.
- Toolbar: show project/workspace context on the left and New terminal, layout and preview actions on the right. Controls should remain available and labeled at narrow widths.
- Pane headers: one compact line for name, status and close; show extra actions on hover, focus or a context menu without hiding their keyboard route. Use straight dividers and normal resize cursors. Highlight drop zones before a drag is released.
- Preserve terminal identity while moving, resizing, tabbing, maximizing and restoring. Grid presets use existing panes and retain overflow as tabs.
- Image preview shares the pane system. Keep Fit, zoom and pan discoverable; scroll-to-zoom acts only over the preview. Missing images need Locate and Retry.

## Supporting flows

- Settings and history remain full main-area views, while terminals continue running. Use shared column edges; stack labels and controls when the window is narrow. Font size reads **− 14 +** without a unit, with its right edge aligned to related controls below.
- New workspace and terminal menus should group folder, launch choice and the committing action. File/folder pickers remain native. Attachments always name the recipient and do not submit the prompt.
- History lists project-scoped OpenCode chats with title search and an unambiguous resume action. Show loading, empty and failure states without importing transcripts into app state.
- Closing: keep Cancel, Run in background and Stop and close together and optically centered. Remember only the action selected with “Don't show this again”; Cancel and dismissal never save a choice. Explicit tray Quit remains an exit action and warns about running processes.
- Errors describe what failed and offer a specific next action. Preserve unavailable references rather than silently discarding them.

## Motion and accessibility

- Keep transitions short and functional: hover feedback, drop-zone indication and pane resizing should respond immediately. Do not animate terminal text or move it during reading. Honor reduced-motion settings.
- Verify text contrast at least 4.5:1 for normal text and 3:1 for large text; ensure focus indicators and actionable boundaries remain visible. Because the backdrop changes, test against actual light and dark wallpapers and provide an opaque fallback when needed.
- Test keyboard-only navigation, visible focus, screen-reader names, Windows high contrast, high DPI, long names and the minimum supported window size. Avoid shrinking terminal text to fit more panes.

## Validation before accepting a design

1. Show an ImageGen preview for significant visual changes and obtain the user's direction before treating it as the target. A generated image is not evidence of application behavior.
2. Capture the real Windows app at normal and high DPI, with light and dark backgrounds. Compare workspace, Settings, creation, history, attachment, image, recovery and close states.
3. Check real resizing, drag/drop, keyboard focus, process continuity and recovery. Test with one and several terminals, long output, and a preview pane.
4. Measure release-build app private bytes against the agreed 200 MiB idle / 350 MiB four-shell-and-image budgets; report hosted processes and GPU allocations separately. Confirm no retained growth in repeated pane and image cycles.
5. Record which checks passed, failed or remain untested. Preserve the existing release-gate caveats until verified.
