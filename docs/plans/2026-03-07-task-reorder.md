# Task Reorder Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove deprecated project reorder behavior and replace it with persisted task reorder in the sidebar.

**Architecture:** Keep workspace-level runtime/actions intact, but introduce task-manifest-backed grouping and ordering for the sidebar. Persist task order in config, reload it on bootstrap/refresh, and handle reorder mode at the sidebar level instead of the projects dialog.

**Tech Stack:** Rust, ftui, Grove config persistence, task manifest discovery

---

### Task 1: Add task-order persistence

**Files:**
- Modify: `src/infrastructure/config.rs`
- Test: `src/infrastructure/config.rs`

**Step 1: Write the failing test**

Add a config round-trip test that saves and reloads `task_order`.

**Step 2: Run test to verify it fails**

Run: `cargo test task_order -- --exact`
Expected: FAIL because config does not persist task order yet.

**Step 3: Write minimal implementation**

Add `task_order: Vec<String>` to the persisted projects-state config and round-trip it through `GroveConfig`.

**Step 4: Run test to verify it passes**

Run: `cargo test task_order -- --exact`
Expected: PASS

### Task 2: Load tasks into the TUI model and sort workspaces by task order

**Files:**
- Modify: `src/ui/tui/model.rs`
- Modify: `src/ui/tui/bootstrap/bootstrap_app.rs`
- Modify: `src/ui/tui/update/update_lifecycle_workspace_refresh.rs`
- Create: `src/ui/tui/tasks.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add a TUI test that bootstraps two tasks with reversed project order and expects sidebar/state ordering to follow task order instead of project order.

**Step 2: Run test to verify it fails**

Run: `cargo test sidebar_uses_task_order_over_project_order -- --exact`
Expected: FAIL because the sidebar/state still follow project order.

**Step 3: Write minimal implementation**

Load task manifests, store them in `GroveApp`, and reorder `state.workspaces` from persisted/manual task order.

**Step 4: Run test to verify it passes**

Run: `cargo test sidebar_uses_task_order_over_project_order -- --exact`
Expected: PASS

### Task 3: Remove project reorder UI and commands

**Files:**
- Modify: `src/ui/tui/commands/catalog.rs`
- Modify: `src/ui/tui/commands/meta.rs`
- Modify: `src/ui/tui/update/update_navigation_commands.rs`
- Modify: `src/ui/tui/update/update_input_key_events.rs`
- Modify: `src/ui/tui/update/update_input_keybinding.rs`
- Modify: `src/ui/tui/dialogs/dialogs_projects_key.rs`
- Modify: `src/ui/tui/dialogs/dialogs_projects_state.rs`
- Modify: `src/ui/tui/dialogs/state.rs`
- Modify: `src/ui/tui/view/view_overlays_projects.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Replace the old project reorder tests with a regression test asserting the projects dialog no longer enters reorder mode on `Ctrl+R`.

**Step 2: Run test to verify it fails**

Run: `cargo test project_dialog_ctrl_r_does_not_enter_reorder_mode -- --exact`
Expected: FAIL because the old reorder path still exists.

**Step 3: Write minimal implementation**

Delete the project reorder state/methods/affordances and remove the palette/help entries.

**Step 4: Run test to verify it passes**

Run: `cargo test project_dialog_ctrl_r_does_not_enter_reorder_mode -- --exact`
Expected: PASS

### Task 4: Add sidebar task reorder mode

**Files:**
- Create: `src/ui/tui/tasks_reorder.rs`
- Modify: `src/ui/tui/view/view_chrome_sidebar/build.rs`
- Modify: `src/ui/tui/update/update_navigation_commands.rs`
- Modify: `src/ui/tui/update/update_input_key_events.rs`
- Modify: `src/ui/tui/view/view_overlays_help/keybind_overlay.rs`
- Modify: `src/ui/tui/view/view_status.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing test**

Add TUI tests for:
- `Ctrl+R` entering task reorder mode
- `j/k` moving whole task groups
- `Enter` saving `task_order`
- `Esc` restoring the previous order

**Step 2: Run test to verify it fails**

Run: `cargo test task_reorder -- --nocapture`
Expected: FAIL because reorder mode does not exist yet.

**Step 3: Write minimal implementation**

Implement in-place task reorder mode on the list pane using the selected task as the moving task.

**Step 4: Run test to verify it passes**

Run: `cargo test task_reorder -- --nocapture`
Expected: PASS

### Task 5: Validate locally

**Files:**
- Modify: `src/ui/tui/mod.rs`
- Modify: `src/infrastructure/config.rs`

**Step 1: Run focused test targets**

Run:
- `cargo test task_order -- --exact`
- `cargo test project_dialog_ctrl_r_does_not_enter_reorder_mode -- --exact`
- `cargo test sidebar_uses_task_order_over_project_order -- --exact`
- `cargo test task_reorder -- --nocapture`

Expected: PASS

**Step 2: Run required local validation**

Run: `make precommit`
Expected: PASS
