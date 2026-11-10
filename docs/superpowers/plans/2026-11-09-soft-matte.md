# Soft Matte Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Apply the approved soft matte finish without adding runtime dependencies.

**Architecture:** Keep existing native egui rendering and custom painting. Share radius and overlay tokens across the shell and icon buttons; preserve terminal geometry.

**Tech Stack:** Rust 1.95, egui 0.36 through eframe, existing Cargo.lock.

**Spec:** ../specs/2026-11-09-soft-matte-design.md (approved in chat).

## Global Constraints

- Controls 6 logical pixels, search/selections 8, floating surfaces 12.
- Static matte surfaces; no blur, new dependencies, or continuous animation.
- Preserve docking, terminal rendering, lifecycle, fonts, and hit areas.
- Full Windows visual and performance acceptance cannot run on this Linux executor.

## Review Focus

- Custom-painted icon buttons must retain keyboard focus outlines.
- Disabled controls must retain coherent corners and disabled feedback.
- Popup frame styles must actually consume the shared radius and shadow values.
- Terminal dividers and pane corners must stay straight.
- Long labels and DPI scaling require Windows visual acceptance; padding must not reduce existing hit areas.

## Task 1: Shared style and surface refinement

Files: src/app.rs, src/app/chrome.rs, src/icons.rs; shared surface tokens in a small module registered by src/main.rs.

- [x] Add shared 6/8/12 corner tokens, neutral fills, and a modest static overlay shadow.
- [x] Apply the tokens to shared widget states (including open menus), popup/dialog frames, custom search fields, navigation/history selections, and icon hover/focus outlines.
- [x] Retain existing code preview/shortcut radii, control dimensions, panel geometry, and focus colors. Keep selected rows distinct from hovered rows.
- [x] Inspect egui frame implementations to confirm floating surfaces consume the configured styles.

## Task 2: Verify and record limits

- [x] Format changed Rust files and run git diff --check.
- [x] Compile the shared styling and painting modules in an external headless egui harness, using the existing dependency versions; exercise rendered shape generation for widgets and floating frames. Do not add tests that merely repeat token constants.
- [x] Run cargo +1.95.0 test --locked -p egui_term -j 4 and node --test plugin/test/core.test.mjs.
- [x] Review the final diff for unchanged hit areas, terminal geometry, dependency files, lifecycle, and keyboard routing.
- [x] Record verified results and outstanding Windows visual/DPI/performance acceptance.

Execution: inline for this small styling change. Existing task environment is already isolated. No new worktree or dependency installation is needed. Reversible visual constants do not warrant implementation-mirroring tests. Leave changes reviewable in the checkout.

## Verification record

- Shared styling and icon modules compiled against egui 0.36.2 in the ignored `.work/soft-matte-check` harness. One headless rendering test passed across 100%, 125%, 150%, and 200% scales, producing tessellated widgets and modal surfaces. This is shape generation, not visual or Windows DPI acceptance.
- Existing terminal widget suite: 9 passed. Plugin suite: 3 passed.
- Rustfmt check and git diff --check passed.
- Independent reviewer found no actionable issues in the product diff. Initial harness API mistakes were corrected to egui 0.36's run_ui/style_of API; unapplied texture deltas are explicitly cleared because the harness has no renderer. The corrected harness passed.
- No manifest or lockfile changes, added runtime dependencies, blur passes, animation loops, or terminal/process behavior changes.
- Full Windows host build, screenshots, focus/DPI acceptance, and performance measurements remain unrun. No claim of measured performance parity.
