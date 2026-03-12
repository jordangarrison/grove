# Real Preview Cursor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace Grove's fake preview `|` cursor with the real ftui frame cursor for interactive agent previews, fixing Claude and improving Codex behavior.

**Architecture:** Keep tmux cursor capture as the source of truth, but stop mutating preview text. Render preview content normally, compute the visible cursor cell in the preview pane, then set ftui's frame cursor at that screen position when the interactive preview owns cursor focus.

**Tech Stack:** Rust, Grove TUI, FrankenTUI (`ftui-render` frame cursor), tmux cursor capture, cargo test, make precommit

---

### Task 1: Replace fake-cursor expectations with real-cursor regression tests

**Files:**
- Modify: `src/ui/tui/mod.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Add focused tests near the existing interactive preview cursor coverage that:

- render Claude interactive preview output
- assert preview text stays `second`, not `s|econd`
- assert the rendered frame has `cursor_position == Some((expected_x, expected_y))`
- assert `cursor_visible == true`

Add the same assertion pattern for Codex.

Add an off-screen test that sets a cursor outside the visible preview range and
asserts `cursor_position.is_none()`.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test interactive_agent_preview_renders_real_cursor_for_claude_in_frame -- --nocapture
```

Expected: FAIL because the preview still injects `|` and does not place the
real frame cursor.

**Step 3: Write minimal implementation**

Update tests to stop asserting text mutation and instead inspect frame cursor
metadata after `with_rendered_frame(...)`.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test interactive_agent_preview_renders_real_cursor_for_claude_in_frame -- --nocapture
cargo test interactive_agent_preview_renders_real_cursor_for_codex_in_frame -- --nocapture
```

Expected: PASS after implementation in later tasks.

**Step 5: Commit**

```bash
git add src/ui/tui/mod.rs
git commit -m "test: cover real preview cursor behavior"
```

### Task 2: Add screen-coordinate mapping for the interactive preview cursor

**Files:**
- Modify: `src/ui/tui/view/view_layout.rs`
- Modify: `src/ui/tui/view/view_preview.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add or refine a test that verifies the preview cursor lands on the second row of
preview output at the captured tmux column after accounting for:

- block border
- preview metadata rows
- preview viewport start

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test interactive_agent_preview_renders_real_cursor_for_claude_in_frame -- --nocapture
```

Expected: FAIL because no screen-position mapping exists yet.

**Step 3: Write minimal implementation**

In `src/ui/tui/view/view_layout.rs`, add a helper with behavior equivalent to:

```rust
pub(super) fn interactive_cursor_screen_position(
    &self,
    preview_inner: Rect,
    preview_height: usize,
) -> Option<(u16, u16)> {
    let (visible_index, cursor_col, cursor_visible) =
        self.interactive_cursor_target(preview_height)?;
    if !cursor_visible {
        return None;
    }

    let output_y = preview_inner.y.saturating_add(PREVIEW_METADATA_ROWS);
    let y = output_y.saturating_add(u16::try_from(visible_index).ok()?);
    let x = preview_inner.x.saturating_add(u16::try_from(cursor_col).ok()?);

    if x >= preview_inner.right() || y >= preview_inner.bottom() {
        return None;
    }

    Some((x, y))
}
```

In `src/ui/tui/view/view_preview.rs`, after rendering preview content, call the
helper and set:

```rust
frame.set_cursor(Some((x, y)));
frame.set_cursor_visible(true);
```

only when the helper returns a visible on-screen position.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test interactive_agent_preview_renders_real_cursor_for_claude_in_frame -- --nocapture
cargo test interactive_agent_preview_renders_real_cursor_for_codex_in_frame -- --nocapture
cargo test interactive_preview_ignores_offscreen_real_cursor -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src/ui/tui/view/view_layout.rs src/ui/tui/view/view_preview.rs src/ui/tui/mod.rs
git commit -m "feat: render preview with real frame cursor"
```

### Task 3: Remove synthetic preview cursor rendering

**Files:**
- Modify: `src/ui/tui/view/view_layout.rs`
- Modify: `src/ui/tui/view/view_preview_content.rs`
- Modify: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add or update an assertion proving preview text remains unchanged for an
interactive preview line where the cursor sits mid-line:

```rust
assert_eq!(
    preview_lines
        .iter()
        .map(ftui::text::Line::to_plain_text)
        .collect::<Vec<_>>(),
    vec!["first".to_string(), "second".to_string(), "third".to_string()],
);
```

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test interactive_agent_preview_renders_real_cursor_for_claude_in_frame -- --nocapture
```

Expected: FAIL because the current path still returns `s|econd`.

**Step 3: Write minimal implementation**

Delete the preview cursor injection path:

- remove `apply_interactive_cursor_overlay_parsed(...)`
- remove `apply_cursor_overlay_to_parsed_line(...)`
- remove dead helper tests that only exist for fake `|` rendering
- stop passing `allow_cursor_overlay: true` through preview text rendering if it
  no longer changes content

Keep only real frame-cursor placement.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test interactive_agent_preview_renders_real_cursor_for_claude_in_frame -- --nocapture
cargo test interactive_agent_preview_renders_real_cursor_for_codex_in_frame -- --nocapture
```

Expected: PASS, with unchanged preview text and visible frame cursor.

**Step 5: Commit**

```bash
git add src/ui/tui/view/view_layout.rs src/ui/tui/view/view_preview_content.rs src/ui/tui/mod.rs
git commit -m "refactor: remove fake preview cursor overlay"
```

### Task 4: Preserve cursor ownership rules

**Files:**
- Modify: `src/ui/tui/view/view.rs`
- Modify: `src/ui/tui/view/view_preview.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add a test proving a focused modal/input widget still owns the cursor even if an
interactive preview session exists.

**Step 2: Run test to verify it fails**

Run:

```bash
cargo test view_modal_cursor_takes_priority_over_preview_cursor -- --nocapture
```

Expected: FAIL if preview rendering overrides a modal-owned cursor.

**Step 3: Write minimal implementation**

Keep the existing default reset in `src/ui/tui/view/view.rs`, but in
`render_preview_pane(...)` only place the preview cursor when:

- no modal is open
- the preview pane is the active interactive surface
- no later overlay render path replaces the cursor

Prefer simple guard conditions over shared mutable cursor ownership state.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test view_modal_cursor_takes_priority_over_preview_cursor -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src/ui/tui/view/view.rs src/ui/tui/view/view_preview.rs src/ui/tui/mod.rs
git commit -m "fix: preserve cursor ownership for overlays"
```

### Task 5: Final verification

**Files:**
- Modify: `src/ui/tui/mod.rs`
- Modify: `src/ui/tui/view/view.rs`
- Modify: `src/ui/tui/view/view_preview.rs`
- Modify: `src/ui/tui/view/view_layout.rs`
- Modify: `src/ui/tui/view/view_preview_content.rs`

**Step 1: Run focused tests**

Run:

```bash
cargo test interactive_agent_preview_renders_real_cursor -- --nocapture
cargo test view_modal_cursor_takes_priority_over_preview_cursor -- --nocapture
```

Expected: PASS.

**Step 2: Run local required validation**

Run:

```bash
make precommit
```

Expected: PASS.

**Step 3: Review for cleanup**

Delete any dead fake-cursor helpers, unused imports, and obsolete tests. No
compatibility layer, no retained `|` fallback.

**Step 4: Commit**

```bash
git add src/ui/tui/mod.rs src/ui/tui/view/view.rs src/ui/tui/view/view_preview.rs src/ui/tui/view/view_layout.rs src/ui/tui/view/view_preview_content.rs
git commit -m "fix: use real cursor in interactive preview"
```
