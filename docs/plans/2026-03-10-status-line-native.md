# Native StatusLine Chrome Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace Grove's custom header/footer chrome layout helper with ftui's native `StatusLine` widget.

**Architecture:** Keep Grove's existing content-generation methods, but swap the rendering primitive from `chrome_bar_line(...)` plus `Paragraph` to `StatusLine` plus native `StatusItem` composition. Preserve hit-region registration and existing footer/header semantics while accepting a flatter visual treatment.

**Tech Stack:** Rust, Grove TUI, FrankenTUI `StatusLine`, targeted cargo tests, `make precommit`

---

### Task 1: Lock the desired header/footer render contract

**Files:**
- Modify: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add/update rendering assertions for:
- header row contains `Grove` and repo name
- command palette visible state still surfaces `Palette` in the header
- footer row still contains `task: ...`, `worktree: ...`, `? help`, and `Ctrl+K palette`

**Step 2: Run test to verify it fails**

Run: `cargo test status_row_ header_`
Expected: at least one test fails once assertions target the new contract

**Step 3: Write minimal implementation**

Do not implement yet, only once the failure is confirmed.

**Step 4: Run test to verify it passes**

Run the same targeted tests after implementation.

**Step 5: Commit**

```bash
git add src/ui/tui/mod.rs
git commit -m "test: lock native status line chrome behavior"
```

### Task 2: Migrate header rendering to `StatusLine`

**Files:**
- Modify: `src/ui/tui/view/view_chrome_header.rs`
- Modify: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Ensure the header contract test fails against the old implementation if needed.

**Step 2: Run test to verify it fails**

Run: `cargo test header_`
Expected: FAIL on the new assertion

**Step 3: Write minimal implementation**

Build the header with `StatusLine::new()`, left `StatusItem::text(...)` items, and shared bar style.

**Step 4: Run test to verify it passes**

Run: `cargo test header_`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/tui/view/view_chrome_header.rs src/ui/tui/mod.rs
git commit -m "refactor: render header with native status line"
```

### Task 3: Migrate footer rendering to `StatusLine`

**Files:**
- Modify: `src/ui/tui/view/view_status.rs`
- Modify: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add/update footer assertions that depend on right-side native key hints and retained context text.

**Step 2: Run test to verify it fails**

Run: `cargo test status_row_`
Expected: FAIL on the new assertion

**Step 3: Write minimal implementation**

Build the footer with:
- left text items for state/context
- right text label for `Keys`
- right native `StatusItem::key_hint("?", "help")`
- right native `StatusItem::key_hint("Ctrl+K", "palette")`

**Step 4: Run test to verify it passes**

Run: `cargo test status_row_`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/tui/view/view_status.rs src/ui/tui/mod.rs
git commit -m "refactor: render footer with native status line"
```

### Task 4: Remove obsolete custom chrome helpers

**Files:**
- Modify: `src/ui/tui/text.rs`
- Delete: `src/ui/tui/text/chrome.rs`
- Modify: any remaining imports in `src/ui/tui/model.rs` or other callers

**Step 1: Write the failing test**

Use compilation and the focused render tests as the regression gate.

**Step 2: Run test to verify it fails**

Run: `cargo test status_row_`
Expected: compile failure or test failure until dead imports/helpers are removed

**Step 3: Write minimal implementation**

Delete unused exports/imports and the dead helper file.

**Step 4: Run test to verify it passes**

Run:
- `cargo test status_row_`
- `cargo test header_`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/tui/text.rs src/ui/tui/model.rs src/ui/tui/view/view_status.rs src/ui/tui/view/view_chrome_header.rs src/ui/tui/mod.rs
git rm src/ui/tui/text/chrome.rs
git commit -m "refactor: remove custom chrome bar helper"
```

### Task 5: Final verification

**Files:**
- Review touched files only

**Step 1: Run focused tests**

Run:
- `cargo test status_row_`
- `cargo test header_`

Expected: PASS

**Step 2: Run required local validation**

Run: `make precommit`
Expected: PASS

**Step 3: Update issue**

Comment on `#51` with the shipped compromise and close it if the migration is complete.

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: replace custom chrome bar with native status line"
```
