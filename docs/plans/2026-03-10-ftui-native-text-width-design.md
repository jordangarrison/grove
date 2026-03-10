# Native ftui Text Width Design

**Issue:** `#55`

**Goal:** Replace Grove's custom text display-width, truncation, padding, and selection-slicing helpers with ftui-native text primitives, and delete the Grove wrappers entirely.

## Constraints

- Prefer direct `ftui::text` usage at call sites over local adapter helpers.
- Keep current user-visible behavior where it matters:
  - width-aware truncation still respects grapheme boundaries
  - row content still fits exact modal and overlay widths
  - preview selection still maps and copies by display cells, not bytes
- Remove log preview truncation for simplicity, log full values instead.
- No backwards compatibility layer, delete the custom helpers once callers move.

## Chosen Approach

Use ftui-native text primitives directly:

- `ftui::text::display_width` for width measurement
- `ftui::text::truncate_with_ellipsis` for UI truncation
- `ftui::text::truncate_to_width` and `truncate_to_width_with_info` for exact-fit slices
- `ftui_text::find_cell_boundary` or `ftui::text::Line::split_at_cell` for cell-aware substring boundaries

This keeps text layout logic anchored in the same library that renders the TUI instead of maintaining a parallel Grove implementation.

## Implementation Notes

- `src/ui/tui/text/visual.rs`
  - delete `line_visual_width`
  - delete `visual_substring`
  - delete `visual_grapheme_at`
  - delete `truncate_to_display_width`
  - delete `pad_or_truncate_to_display_width`
  - delete `truncate_for_log`
- `src/ui/tui/text.rs`
  - remove the `visual` module re-exports
- `src/ui/tui/model.rs`
  - remove imports of deleted helpers
- UI truncation and padding callers
  - `src/ui/tui/dialogs/dialogs.rs`
  - `src/ui/tui/logging/logging_state.rs`
  - `src/ui/tui/view/view_overlays_confirm.rs`
  - `src/ui/tui/view/view_overlays_create.rs`
  - `src/ui/tui/view/view_overlays_edit.rs`
  - `src/ui/tui/view/view_overlays_projects.rs`
  - `src/ui/tui/view/view_overlays_rename_tab.rs`
  - `src/ui/tui/view/view_overlays_session_cleanup.rs`
  - `src/ui/tui/view/view_overlays_settings.rs`
  - `src/ui/tui/view/view_overlays_workspace_delete.rs`
  - `src/ui/tui/view/view_overlays_workspace_launch.rs`
  - `src/ui/tui/view/view_overlays_workspace_merge.rs`
  - `src/ui/tui/view/view_overlays_workspace_stop.rs`
  - `src/ui/tui/view/view_overlays_workspace_update.rs`
- Selection and logging callers
  - `src/ui/tui/view/view_selection_interaction.rs`
  - `src/ui/tui/view/view_selection_logging.rs`

## Testing

Add regression coverage for:

- width-aware truncation with ellipsis for ASCII, CJK, emoji, and combining marks
- padding behavior that preserves exact row width after truncation
- cell-aware preview selection slicing across wide graphemes
- grapheme metadata logging around a display-cell position
- logging now emits full values instead of truncated previews

## Non-Goals

- changing copy-selection semantics
- changing telemetry schema names
- introducing a new Grove text abstraction on top of ftui
