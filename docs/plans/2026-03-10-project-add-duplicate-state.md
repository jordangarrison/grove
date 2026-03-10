# Project Add Duplicate State Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show already-added repos inline in the Add Project search results and make those rows non-accepting.

**Architecture:** Extend stored path matches with duplicate state derived from configured projects, render rows with ftui rich text for inline status, and gate keyboard and mouse acceptance on that duplicate bit while preserving the final add-time validation.

**Tech Stack:** Rust, ftui `List`/`ListItem`, ftui text spans, focused TUI tests in `src/ui/tui/mod.rs`, existing duplicate-path detection via `refer_to_same_location`.

---

### Task 1: Add failing regressions for duplicate result UX

**Files:**
- Modify: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

- Add tests covering:
  - duplicate repo matches render `Already added`
  - Enter on a duplicate result does not fill the path field or move focus
  - clicking a duplicate result does not accept it

**Step 2: Run test to verify it fails**

Run: `cargo test project_add_dialog_duplicate -- --nocapture`

Expected: FAIL because duplicate rows are currently rendered as normal matches and accepted.

**Step 3: Write minimal implementation**

- none, test-only step

**Step 4: Run test to verify it passes**

Run: `cargo test project_add_dialog_duplicate -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/tui/mod.rs
git commit -m "test: cover duplicate add-project search matches"
```

### Task 2: Implement duplicate row state and rendering

**Files:**
- Modify: `src/ui/tui/dialogs/state.rs`
- Modify: `src/ui/tui/dialogs/dialogs_projects_search.rs`
- Modify: `src/ui/tui/view/view_overlays_projects.rs`
- Modify: `src/ui/tui/update/update_input_mouse.rs`

**Step 1: Write the failing test**

- Re-run the Task 1 regressions.

**Step 2: Run test to verify it fails**

Run: `cargo test project_add_dialog_duplicate -- --nocapture`

Expected: FAIL

**Step 3: Write minimal implementation**

- Add `already_added` to `ProjectPathMatch`.
- Mark duplicate matches during refresh.
- Render duplicate rows with muted styling and inline `Already added` text.
- Prevent duplicate row acceptance in keyboard and mouse flows.

**Step 4: Run test to verify it passes**

Run: `cargo test project_add_dialog_duplicate -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/tui/dialogs/state.rs src/ui/tui/dialogs/dialogs_projects_search.rs src/ui/tui/view/view_overlays_projects.rs src/ui/tui/update/update_input_mouse.rs src/ui/tui/mod.rs
git commit -m "feat: show duplicate add-project matches inline"
```
