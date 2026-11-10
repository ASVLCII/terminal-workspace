# Soft matte interface refinement

Status: approved in chat; implemented in the checkout. Windows visual acceptance remains outstanding.

## Intent

Give Terminal Workspace a mostly matte, precise finish inspired by the user's Linear and Notion references. Improve the existing native Rust/egui shell without adding features or dependencies.

## Approaches

The user selected soft matte: moderate rounding, quiet surface depth, and crisp terminal boundaries. Strict matte would retain smaller corners and almost no shadows, but would produce less visible improvement. True glass blur was considered and rejected in favor of the user's mostly matte preference and low overhead requirement.

## Scope and decisions

- Centralize shell corner and surface values beside the existing chrome palette.
- Use 6 logical pixel corners for ordinary controls, 8 for search fields and selected navigation rows, and 12 for floating menus and dialogs. Existing straight terminal splits and pane geometry remain intact.
- Give floating surfaces a subtle static shadow and thin neutral edge. Main surfaces stay opaque; no blur, glow, translucent terminal background, or continuous animation.
- Keep graphite surfaces distinct but close in tone. Preserve readable text and explicit hover and keyboard focus treatment.
- Normalize padding for affected controls on the existing 4-unit spacing scale. Preserve current layout, font choices, terminal density, and control hit areas.
- Apply consistently to shared widget styling, custom search frames, sidebar selection painting, menus, and dialogs. Do not round informational code samples or shortcut outlines merely because they currently share a literal radius.

These dimensions and the decision to use static feedback are implementation proposals, not independently user-specified measurements.

## Limits

No new crates, assets, settings, background services, custom shaders, or features. No changes to terminal rendering, process lifecycle, saved data, keyboard routing, or docking behavior. Existing design rules prohibiting ornamental glow and gradients remain in force.

## Validation

Inspect the diff and formatting; run the portable Rust widget and Node plugin suites. Check changed egui APIs against the installed locked dependency sources. Full host compilation and interactive visual, focus, DPI, and performance acceptance require Windows; this Linux environment cannot establish those results. Report that limitation explicitly rather than representing portable tests as visual validation.

## Completion

A small, reviewable shell styling change with centralized values, consistent surface treatment, no added dependencies, and verification results stated with their platform limits. Windows screenshots at normal and high DPI remain required before claiming visual acceptance or measured performance parity.
