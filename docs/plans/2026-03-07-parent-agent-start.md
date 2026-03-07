# Parent Agent Start Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an explicit `Start Parent Agent` action on `Task Home` without changing `a`, so users can launch the task-root parent session and still create workspace agent tabs reliably.

**Architecture:** Add a new `UiCommand` dedicated to parent-agent start, wire it into palette/keybinding/help visibility, and route it to a task-root-scoped launch dialog plus existing tmux launch service. Keep parent-session stop/preview behavior unchanged. Implement via TDD, starting from command availability and launch-path regressions.

**Tech Stack:** Rust, Grove TUI command system, tmux runtime service, unit tests in `src/ui/tui/mod.rs`

---

### Task 1: Add Parent-Agent Command Surface

**Files:**
- Modify: `src/ui/tui/commands/catalog.rs`
- Modify: `src/ui/tui/commands/meta.rs`
- Modify: `src/ui/tui/update/update_navigation_commands.rs`
- Modify: `src/ui/tui/update/update_navigation_palette.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Add tests proving:
- `A` on `Task Home` maps to a new `StartParentAgent` command.
- Command palette includes `Start Parent Agent` on `Task Home`.
- Command palette excludes or disables it off `Task Home`.

Suggested test names:

```rust
fn task_home_keybind_a_upper_triggers_start_parent_agent()
fn command_palette_lists_start_parent_agent_on_task_home()
fn command_palette_hides_start_parent_agent_off_task_home()
```

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test task_home_keybind_a_upper_triggers_start_parent_agent
cargo test command_palette_lists_start_parent_agent_on_task_home
cargo test command_palette_hides_start_parent_agent_off_task_home
```

Expected: FAIL with missing command metadata or wrong palette contents.

**Step 3: Write minimal implementation**

Add a new command variant and metadata:

```rust
enum UiCommand {
    StartParentAgent,
    // existing variants...
}
```

Add palette/keybinding/help metadata with:

```rust
title: "Start Parent Agent"
description: "Open the task-root parent agent launch dialog (A)"
```

Handle it in command execution by dispatching to a new parent-agent launch entry
point.

**Step 4: Run tests to verify they pass**

Run the same three commands.

Expected: PASS.

**Step 5: Commit**

```bash
git add src/ui/tui/commands/catalog.rs src/ui/tui/commands/meta.rs src/ui/tui/update/update_navigation_commands.rs src/ui/tui/update/update_navigation_palette.rs src/ui/tui/mod.rs
git commit -m "feat: add parent agent command surface"
```

### Task 2: Add Task-Root Launch Dialog Path

**Files:**
- Modify: `src/ui/tui/dialogs/dialogs_launch.rs`
- Modify: `src/ui/tui/update/update_lifecycle_start.rs`
- Modify: `src/ui/tui/model.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Add tests proving:
- `A` on `Task Home` opens a launch dialog seeded from task-root settings.
- Confirming that dialog launches `grove-task-<task-slug>` in the task root.
- `a` on `Task Home` still launches `grove-wt-<task>-<repo>-agent-1`.
- Duplicate parent session yields `parent agent already running`.

Suggested test names:

```rust
fn task_home_start_parent_agent_opens_task_scoped_launch_dialog()
fn task_home_start_parent_agent_launches_task_root_session()
fn task_home_start_parent_agent_duplicate_session_shows_already_running_toast()
fn task_home_new_agent_tab_still_launches_selected_workspace_agent_tab()
```

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test task_home_start_parent_agent_opens_task_scoped_launch_dialog
cargo test task_home_start_parent_agent_launches_task_root_session
cargo test task_home_start_parent_agent_duplicate_session_shows_already_running_toast
cargo test task_home_new_agent_tab_still_launches_selected_workspace_agent_tab
```

Expected: FAIL because no dedicated parent-agent path exists yet.

**Step 3: Write minimal implementation**

Reintroduce a dedicated task-root launch path, but only behind the new command.

Example shape:

```rust
fn open_start_parent_agent_dialog(&mut self) { /* use task.root_path markers */ }

fn confirm_start_parent_agent_dialog(&mut self) { /* launch grove-task-<slug> */ }
```

Keep the regular `open_start_dialog` / `confirm_start_dialog` path workspace-scoped.
On duplicate-session error, convert the result into an info toast:

```rust
"parent agent already running"
```

**Step 4: Run tests to verify they pass**

Run the same four commands.

Expected: PASS.

**Step 5: Commit**

```bash
git add src/ui/tui/dialogs/dialogs_launch.rs src/ui/tui/update/update_lifecycle_start.rs src/ui/tui/model.rs src/ui/tui/mod.rs
git commit -m "feat: add parent agent launch flow"
```

### Task 3: Update Task Home Discoverability

**Files:**
- Modify: `src/ui/tui/update/update_navigation_preview.rs`
- Modify: `src/ui/tui/view/view_overlays_help/keybind_overlay.rs`
- Modify: `src/ui/tui/commands/meta.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Add tests proving:
- `Task Home` splash mentions `A` for parent agent and `a` for workspace tabs.
- Keybind help includes `A start parent agent`.
- Help still includes `a new agent tab`.

Suggested test names:

```rust
fn task_home_splash_mentions_parent_agent_keybind()
fn keybind_help_lists_start_parent_agent()
fn keybind_help_keeps_new_agent_tab_hint()
```

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test task_home_splash_mentions_parent_agent_keybind
cargo test keybind_help_lists_start_parent_agent
cargo test keybind_help_keeps_new_agent_tab_hint
```

Expected: FAIL because current copy only mentions `a`.

**Step 3: Write minimal implementation**

Update `task_home_splash()` copy to separate the actions:

```text
Press 'A' to start parent agent.
Then use 'a' for workspace agent tabs, 's' for shell tabs, 'g' for git tab.
```

Expose the new help hint in command metadata so the help overlay renders it via
the existing hint pipeline.

**Step 4: Run tests to verify they pass**

Run the same three commands.

Expected: PASS.

**Step 5: Commit**

```bash
git add src/ui/tui/update/update_navigation_preview.rs src/ui/tui/view/view_overlays_help/keybind_overlay.rs src/ui/tui/commands/meta.rs src/ui/tui/mod.rs
git commit -m "feat: expose parent agent discoverability"
```

### Task 4: Full Validation

**Files:**
- Modify: none
- Test: `src/ui/tui/mod.rs`

**Step 1: Run focused regression tests**

Run:

```bash
cargo test task_home_start_parent_agent
cargo test task_home_new_agent_tab_still_launches_selected_workspace_agent_tab
cargo test start_dialog_launches_numbered_agent_session_for_task_worktree
```

Expected: PASS.

**Step 2: Run required local validation**

Run:

```bash
make precommit
```

Expected: `cargo fmt --check`, `cargo check`, and `cargo clippy` all PASS.

**Step 3: Review UX manually**

Check:
- `Task Home` shows the new hint text.
- `A` opens parent-agent launch dialog.
- `a` still opens workspace-agent tab dialog.
- Palette shows both actions in the right context.

**Step 4: Commit final integration**

```bash
git add src/ui/tui/commands/catalog.rs src/ui/tui/commands/meta.rs src/ui/tui/dialogs/dialogs_launch.rs src/ui/tui/update/update_lifecycle_start.rs src/ui/tui/update/update_navigation_commands.rs src/ui/tui/update/update_navigation_palette.rs src/ui/tui/update/update_navigation_preview.rs src/ui/tui/view/view_overlays_help/keybind_overlay.rs src/ui/tui/model.rs src/ui/tui/mod.rs
git commit -m "feat: add explicit parent agent start flow"
```
