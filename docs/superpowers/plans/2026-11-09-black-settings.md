# Black Settings implementation plan

Approved target: generated_images/exec-51077d14-3e1b-4865-a0f3-dc727b193505.png (latest revision of A). The user approved this image after requesting the size changer align with the common right content edge.

- Preserve existing published changes; no environment draft updates or publishing.
- Use achromatic shell palette, retaining semantic status/error colors and terminal-owned palette.
- Extract Settings into src/app/settings.rs; maintain font loading, scrollback updates, save callbacks and closing choices.
- Apply title/subtitle, terminal sample, Font/Size row, right-aligned behavior controls and keycaps. Preserve real shortcut bindings instead of invented mockup bindings.
- Stack form groups at narrow widths and maintain readable text.
- Validate actual Settings code using a headless harness with host-service stubs, test/render wide and narrow layouts, run existing portable suites and formatting.
- Review diff and document that full Windows build, real app screenshots and performance acceptance remain unavailable here.

## Results

Implemented Settings in src/app/settings.rs and achromatic shell tokens in src/surfaces.rs. Existing runtime handlers retained. Real shortcuts preserved instead of ImageGen's unverified Maximize binding. Sidebar hierarchy and existing navigation behavior retained rather than reducing multiple-project support to the single-project mockup.

Actual Settings module compiles in the ignored headless harness with host-service stubs. Two harness tests pass: component tessellation at multiple scales; actual Settings at widths 400, 540, 640, 920 and 1100 with no overflow, no passive preference writes and aligned behavior dropdown columns. Initial narrow overflow and visually drifting columns were caught and corrected. Rendered Settings inspected in artifacts/soft-matte/preview.png; its surrounding sidebar is illustrative harness content, not an integrated Windows capture.

Portable suites: 9 Rust widget tests and 3 Node plugin tests passed. Formatting and diff checks passed. Independent review found no actionable issues in extracted behavior. No dependency/lockfile changes, new services or animation loops. Environment settings were not updated or published.

Full Windows build, integrated visual/focus/DPI acceptance and performance measurements remain unrun. Visual acceptance remains blocked until a real Windows capture can be compared with the approved image.
