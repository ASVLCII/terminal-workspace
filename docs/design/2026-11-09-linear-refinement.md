# Linear-inspired refinement

Date: 2026-11-09. Scope: refinement of the implemented Windows terminal workspace.

The user requested improvements based on [Linear](https://linear.app/). Inspected the live public homepage, its embedded product illustration, and computed body/navigation styles. This was not an authenticated Linear application inspection. Local reference captures are under ignored artifacts/.

Apply the product illustration's compact hierarchy, restrained borders, muted metadata, small navigation labels and subtle selected rows. Preserve the approved workspace structure, pinned Settings footer, straight resizable panes, image pane, centered confirmation actions and spaced shortcut rows. Keep the existing Windows UI font and CaskaydiaMono NFM terminal font. This refinement adds no dependencies, animations or new product features.

## Current tokens and proportions

- Canvas #151516; sidebar #18191C; toolbar/menu surface #1A1B1E.
- Border #303238; selected row #2D2F36; hover #24262B.
- Primary text #E8E9ED; secondary text #9EA1AB. Terminal colors remain controlled by the terminal palette.
- Toolbar 44 logical pixels, pane tabs 34, footer 36. Compact 13 px navigation and pane titles, 12 px supporting status; terminal font size remains independently adjustable.
- Settings title 22 px; section headings 14 px. Shortcut labels align to the left and key outlines retain their generous vertical spacing.
- Icon buttons retain accessible names and now display explicit hover and keyboard focus boundaries. Active/focused tabs receive distinct neutral surfaces.

The original boards remain the behavior and layout authority; these tokens supersede their proposed initial palette and chrome measurements. Reference screenshots are visual evidence only. Functional, performance and accessibility verification is recorded separately in docs/IMPLEMENTATION.md.
