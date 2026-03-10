# Native ftui Text Width Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace Grove's custom text width, truncation, padding, and selection helpers with direct ftui text APIs, then delete the Grove helper module.

**Architecture:** Move width-aware UI rendering to `ftui::text` primitives at each caller and move cell-boundary selection logic to ftui-native boundary helpers. Treat the existing Grove helpers as dead code once all callers migrate, including log-preview truncation.

**Tech Stack:** Rust, Grove TUI, FrankenTUI `ftui::text`, targeted `cargo test`, `make precommit`

---

### Task 1: Lock truncation and padding behavior with focused tests

**Files:**
- Modify: `src/ui/tui/dialogs/dialogs.rs`
- Modify: `src/ui/tui/logging/logging_state.rs`
- Test: `src/ui/tui/dialogs/dialogs.rs`
- Test: `src/ui/tui/logging/logging_state.rs`

**Step 1: Write the failing test**

Add `#[cfg(test)]` coverage for:
- modal input rows truncate with ellipsis and still fit `content_width`
- modal actions rows pad to exact width after truncation
- logging state message normalization still returns a string whose display width does not exceed the limit
- Unicode cases include CJK and emoji

Example assertions:

```rust
assert_eq!(ftui::text::display_width(rendered.as_str()), content_width);
assert!(rendered.contains('…'));
assert!(ftui::text::display_width(message.as_str()) <= max_message_width);
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test modal_labeled_input_row
cargo test trim_status_message
```

Expected: FAIL until the callers switch to ftui-native truncation and padding.

**Step 3: Write minimal implementation**

Do not add helpers. Inline `ftui::text::truncate_with_ellipsis` and width-based padding logic in the production callers once the failures are confirmed.

**Step 4: Run test to verify it passes**

Run the same targeted tests.

**Step 5: Commit**

```bash
git add src/ui/tui/dialogs/dialogs.rs src/ui/tui/logging/logging_state.rs
git commit -m "test: lock ftui text width behavior"
```

### Task 2: Replace UI truncation and padding callers with direct `ftui::text`

**Files:**
- Modify: `src/ui/tui/dialogs/dialogs.rs`
- Modify: `src/ui/tui/logging/logging_state.rs`
- Modify: `src/ui/tui/view/view_overlays_confirm.rs`
- Modify: `src/ui/tui/view/view_overlays_create.rs`
- Modify: `src/ui/tui/view/view_overlays_edit.rs`
- Modify: `src/ui/tui/view/view_overlays_projects.rs`
- Modify: `src/ui/tui/view/view_overlays_rename_tab.rs`
- Modify: `src/ui/tui/view/view_overlays_session_cleanup.rs`
- Modify: `src/ui/tui/view/view_overlays_settings.rs`
- Modify: `src/ui/tui/view/view_overlays_workspace_delete.rs`
- Modify: `src/ui/tui/view/view_overlays_workspace_launch.rs`
- Modify: `src/ui/tui/view/view_overlays_workspace_merge.rs`
- Modify: `src/ui/tui/view/view_overlays_workspace_stop.rs`
- Modify: `src/ui/tui/view/view_overlays_workspace_update.rs`

**Step 1: Write the failing test**

If any call site is still uncovered, add one focused assertion per rendering pattern:
- static row truncation
- title row exact-width padding
- project path truncation in overlays

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test modal_
cargo test overlay_
```

Expected: at least one failure or compile error while the old helper imports still exist.

**Step 3: Write minimal implementation**

At each call site:
- replace `truncate_to_display_width(value, width)` with `ftui::text::truncate_with_ellipsis(value, width, "…")`
- replace `pad_or_truncate_to_display_width(...)` with direct truncation plus `" ".repeat(width.saturating_sub(ftui::text::display_width(...)))`
- keep the code local, no new shared helper

**Step 4: Run test to verify it passes**

Run the same targeted tests.

**Step 5: Commit**

```bash
git add src/ui/tui/dialogs/dialogs.rs src/ui/tui/logging/logging_state.rs src/ui/tui/view/view_overlays_confirm.rs src/ui/tui/view/view_overlays_create.rs src/ui/tui/view/view_overlays_edit.rs src/ui/tui/view/view_overlays_projects.rs src/ui/tui/view/view_overlays_rename_tab.rs src/ui/tui/view/view_overlays_session_cleanup.rs src/ui/tui/view/view_overlays_settings.rs src/ui/tui/view/view_overlays_workspace_delete.rs src/ui/tui/view/view_overlays_workspace_launch.rs src/ui/tui/view/view_overlays_workspace_merge.rs src/ui/tui/view/view_overlays_workspace_stop.rs src/ui/tui/view/view_overlays_workspace_update.rs
git commit -m "refactor: replace custom ui text width helpers"
```

### Task 3: Lock preview selection cell-boundary behavior with tests

**Files:**
- Modify: `src/ui/tui/view/view_selection_interaction.rs`
- Modify: `src/ui/tui/view/view_selection_logging.rs`
- Modify: `src/ui/tui/mod.rs`
- Test: `src/ui/tui/view/view_selection_interaction.rs`
- Test: `src/ui/tui/view/view_selection_logging.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add regression tests for:
- single-line and multi-line selection slicing by display cells
- slicing around wide graphemes and combining marks
- grapheme metadata logging at a selected display column
- full, untruncated logging payloads for selection previews

Example assertions:

```rust
assert_eq!(selected, vec!["你a".to_string()]);
assert_eq!(event_data["grapheme"], "👩‍🔬");
assert_eq!(event_data["line_context"], full_context);
```

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test selected_preview_text_lines
cargo test add_selection_point_snapshot_fields
```

Expected: FAIL until selection slicing and logging switch to ftui-native boundary calculations and full-value logging.

**Step 3: Write minimal implementation**

Use ftui-native cell-boundary logic directly:
- compute byte start and end with `ftui_text::find_cell_boundary`
- slice the original line with those byte offsets
- compute widths with `ftui::text::display_width`
- derive grapheme-at-column metadata from ftui grapheme iteration or boundary helpers
- remove all log preview truncation and write full strings into the event data

**Step 4: Run test to verify it passes**

Run the same targeted tests.

**Step 5: Commit**

```bash
git add src/ui/tui/view/view_selection_interaction.rs src/ui/tui/view/view_selection_logging.rs src/ui/tui/mod.rs
git commit -m "refactor: move selection text handling to ftui"
```

### Task 4: Delete the obsolete helper module and dead imports

**Files:**
- Modify: `src/ui/tui/text.rs`
- Modify: `src/ui/tui/model.rs`
- Delete: `src/ui/tui/text/visual.rs`

**Step 1: Write the failing test**

Use compilation and the focused tests from Tasks 1 through 3 as the regression gate.

**Step 2: Run test to verify it fails**

Run:
```bash
cargo test modal_
cargo test selected_preview_text_lines
```

Expected: compile failures or unresolved imports until the deleted helper references are removed.

**Step 3: Write minimal implementation**

- remove `mod visual;`
- remove re-exports of deleted helpers
- remove stale imports from `src/ui/tui/model.rs` and any remaining callers
- delete `src/ui/tui/text/visual.rs`

**Step 4: Run test to verify it passes**

Run:
```bash
cargo test modal_
cargo test selected_preview_text_lines
cargo test add_selection_point_snapshot_fields
```

Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/tui/text.rs src/ui/tui/model.rs src/ui/tui/view/view_selection_interaction.rs src/ui/tui/view/view_selection_logging.rs src/ui/tui/dialogs/dialogs.rs src/ui/tui/logging/logging_state.rs src/ui/tui/mod.rs
git rm src/ui/tui/text/visual.rs
git commit -m "refactor: delete custom text visual helpers"
```

### Task 5: Final verification

**Files:**
- Review touched files only

**Step 1: Run focused tests**

Run:
```bash
cargo test modal_
cargo test trim_status_message
cargo test selected_preview_text_lines
cargo test add_selection_point_snapshot_fields
```

Expected: PASS

**Step 2: Run required local validation**

Run:
```bash
make precommit
```

Expected: PASS

**Step 3: Inspect for dead code**

Run:
```bash
rg -n "truncate_to_display_width|pad_or_truncate_to_display_width|visual_substring|visual_grapheme_at|line_visual_width|truncate_for_log" src
```

Expected: no matches

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: replace custom text helpers with native ftui"
```
