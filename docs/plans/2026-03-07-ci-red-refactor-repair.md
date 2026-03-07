# CI Red Refactor Repair Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restore `make ci` to green by finishing the task/worktree refactor, re-establishing state invariants, and removing legacy workspace-mode behavior so Grove matches the end-state task-first architecture.

**Architecture:** Fix this in layers. First restore `AppState` invariants so selection cannot drift. Then remove legacy workspace/bootstrap paths that still treat project/workspace discovery as a first-class mode. After that, complete the tmux session naming split so task roots and task worktrees use the task-first runtime model everywhere. Finish by updating the affected UI tests and running the required validation.

**Tech Stack:** Rust, ftui, tmux runtime integration, Grove task discovery, Grove TUI tests

---

**Decisions locked before execution:**
- Grove should implement the end-state task/worktree model only. Legacy project/workspace compatibility is out of scope.
- Task worktree runtime paths should use `grove-wt-*` session names end-to-end. Discovery and cleanup already encode that contract.

### Task 1: Restore `AppState` selection invariants

**Files:**
- Modify: `src/ui/state.rs`
- Modify: `src/ui/tui/update/update_input_mouse.rs`
- Modify: `src/ui/tui/update/update_lifecycle_workspace_refresh.rs`
- Modify: `src/ui/tui/mod.rs`
- Test: `src/ui/state.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Add focused regressions for:
- selecting a workspace by flat index updates `selected_index`, `selected_task_index`, and `selected_worktree_index` together
- mouse workspace selection updates task/worktree selection too
- refresh completion restoring a selected workspace path also restores task/worktree selection

**Step 2: Run tests to verify they fail**

Run:
- `cargo test reducer_moves_selection_with_bounds -- --exact`
- `cargo test mouse_workspace_selection_uses_row_hit_data_after_render -- --exact`
- `cargo test mouse_click_on_list_selects_workspace -- --exact`

Expected: FAIL because selection metadata drifts when only `selected_index` changes.

**Step 3: Write minimal implementation**

Add a single `AppState` API for selecting/restoring by flat index and use it everywhere instead of assigning `selected_index` directly. Keep `sync_selection_fields()` internal and make mouse/refresh/replay paths go through the same invariant-preserving entrypoint.

**Step 4: Run tests to verify they pass**

Run:
- `cargo test reducer_moves_selection_with_bounds -- --exact`
- `cargo test mouse_workspace_selection_uses_row_hit_data_after_render -- --exact`
- `cargo test mouse_click_on_list_selects_workspace -- --exact`

Expected: PASS

### Task 2: Remove legacy workspace-mode bootstrap and state reconstruction

**Files:**
- Modify: `src/ui/tui/tasks.rs`
- Modify: `src/ui/tui/bootstrap/bootstrap_app.rs`
- Modify: `src/ui/tui/mod.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Add regressions for:
- task bootstrap keeps task/worktree identity stable after task-order reconciliation
- no codepath rebuilds state from legacy `BootstrapData.workspaces`
- shell/sidebar rendering continues to show task/worktree rows from manifest-backed task state

**Step 2: Run tests to verify they fail**

Run:
- `cargo test shell_contains_list_preview_and_status_placeholders -- --exact`
- `cargo test selected_workspace_row_has_selection_marker -- --exact`
- `cargo test shell_lines_show_workspace_and_agent_labels_without_status_badges -- --exact`

Expected: FAIL because legacy workspace-shaped bootstrap/state reconstruction still leaks into the task-first TUI path.

**Step 3: Write minimal implementation**

Delete the legacy workspace-shaped reconstruction path from the TUI bootstrap/reorder flow. Keep task ordering and selection restoration entirely task/worktree-native and remove any remaining dependency on `AppState::from_workspaces(...)` from the runtime TUI path.

**Step 4: Run tests to verify they pass**

Run:
- `cargo test shell_contains_list_preview_and_status_placeholders -- --exact`
- `cargo test selected_workspace_row_has_selection_marker -- --exact`
- `cargo test shell_lines_show_workspace_and_agent_labels_without_status_badges -- --exact`

Expected: PASS

### Task 3: Make startup and refresh task-first only

**Files:**
- Modify: `src/ui/tui/bootstrap/bootstrap_app.rs`
- Modify: `src/ui/tui/update/update_lifecycle_workspace_refresh.rs`
- Modify: `src/infrastructure/paths.rs`
- Modify: `src/ui/tui/mod.rs`
- Test: `src/ui/tui/mod.rs`
- Test: `src/infrastructure/paths.rs`

**Step 1: Write the failing tests**

Add regressions for:
- startup bootstraps from task manifests only
- manual refresh reloads task manifests instead of project discovery
- no runtime path depends on legacy workspace discovery

**Step 2: Run tests to verify they fail**

Run:
- `cargo test uppercase_r_refreshes_workspaces_from_list_mode -- --exact`
- `cargo test uppercase_r_is_debounced_after_recent_manual_refresh -- --exact`
- `cargo test manual_refresh_completion_shows_success_toast -- --exact`
- `cargo test manual_refresh_completion_shows_error_toast -- --exact`

Expected: FAIL because refresh/startup still contain split logic between task manifests and legacy workspace discovery.

**Step 3: Write minimal implementation**

Remove the legacy project/workspace bootstrap branch from startup and refresh. Make task manifest discovery the only runtime source of truth for the TUI.

**Step 4: Run tests to verify they pass**

Run:
- `cargo test uppercase_r_refreshes_workspaces_from_list_mode -- --exact`
- `cargo test uppercase_r_is_debounced_after_recent_manual_refresh -- --exact`
- `cargo test manual_refresh_completion_shows_success_toast -- --exact`
- `cargo test manual_refresh_completion_shows_error_toast -- --exact`

Expected: PASS

### Task 4: Complete tmux session naming for task worktrees

**Files:**
- Modify: `src/application/agent_runtime/sessions.rs`
- Modify: `src/application/agent_runtime/polling.rs`
- Modify: `src/application/agent_runtime/execution.rs`
- Modify: `src/application/agent_runtime/restart.rs`
- Modify: `src/ui/tui/update/update_navigation_preview.rs`
- Modify: `src/ui/tui/update/update_polling_capture_dispatch.rs`
- Modify: `src/ui/tui/update/update_polling_capture_live.rs`
- Modify: `src/ui/tui/update/update_input_interactive_send.rs`
- Modify: `src/ui/tui/update/update_lifecycle_stop.rs`
- Modify: `src/ui/tui/update/update_navigation_tabs.rs`
- Modify: `src/ui/tui/mod.rs`
- Test: `src/application/agent_runtime/mod.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Add focused regressions proving:
- task worktree sessions use `grove-wt-<task>-<repo>` where appropriate
- task root parent-agent sessions use `grove-task-<task>`
- interactive send, stop, resize, git, shell, and polling targets choose the correct task-first family of session names

**Step 2: Run tests to verify they fail**

Run:
- `cargo test interactive_keys_forward_to_tmux_session -- --exact`
- `cargo test interactive_shift_tab_forwards_to_tmux_session -- --exact`
- `cargo test interactive_shift_enter_forwards_to_tmux_session -- --exact`
- `cargo test git_tab_launches_lazygit_with_dedicated_tmux_session -- --exact`
- `cargo test enter_on_git_tab_attaches_to_lazygit_session -- --exact`
- `cargo test async_preview_polls_workspace_status_targets_when_live_preview_missing -- --exact`

Expected: FAIL because legacy workspace-scoped naming helpers are still used in task-first runtime paths.

**Step 3: Write minimal implementation**

Add a canonical helper for “runtime session identity for selected item” and route task worktree paths through `grove-wt-*` and task-root parent-agent paths through `grove-task-*`. Remove direct uses of workspace-scoped session naming from the task-first runtime path.

**Step 4: Run tests to verify they pass**

Run:
- `cargo test interactive_keys_forward_to_tmux_session -- --exact`
- `cargo test interactive_shift_tab_forwards_to_tmux_session -- --exact`
- `cargo test interactive_shift_enter_forwards_to_tmux_session -- --exact`
- `cargo test git_tab_launches_lazygit_with_dedicated_tmux_session -- --exact`
- `cargo test enter_on_git_tab_attaches_to_lazygit_session -- --exact`
- `cargo test async_preview_polls_workspace_status_targets_when_live_preview_missing -- --exact`

Expected: PASS

### Task 5: Reconcile sidebar, preview, and dialog behavior with the repaired model

**Files:**
- Modify: `src/ui/tui/view/view_chrome_sidebar/build.rs`
- Modify: `src/ui/tui/view/view_preview_shell.rs`
- Modify: `src/ui/tui/view/view_preview_content.rs`
- Modify: `src/ui/tui/update/update_lifecycle_start.rs`
- Modify: `src/ui/tui/dialogs/dialogs_launch.rs`
- Modify: `src/ui/tui/dialogs/dialogs_stop.rs`
- Modify: `src/ui/tui/dialogs/dialogs_edit.rs`
- Modify: `src/ui/tui/dialogs/dialogs_merge.rs`
- Modify: `src/ui/tui/dialogs/dialogs_update_from_base.rs`
- Modify: `src/ui/tui/mod.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write the failing tests**

Use the existing red tests as the target set for:
- main-workspace home/start/stop behavior
- sidebar attention, PR metadata, selection block, and color rows
- merge/update/edit dialogs targeting the actual selected workspace
- preview summary and waiting/active/done/error status rendering

**Step 2: Run tests to verify they fail**

Run:
- `cargo test start_key_opens_dialog_for_main_workspace -- --exact`
- `cargo test stop_key_on_active_main_workspace_stops_agent -- --exact`
- `cargo test merge_key_opens_merge_dialog_for_selected_workspace -- --exact`
- `cargo test update_key_opens_update_from_base_dialog_for_selected_workspace -- --exact`
- `cargo test workspace_pr_token_registers_link_hit_data -- --exact`
- `cargo test waiting_workspace_row_has_no_status_badge_or_input_banner -- --exact`

Expected: FAIL until the repaired model is threaded through the UI behavior.

**Step 3: Write minimal implementation**

Remove assumptions that `selected_task()` and `selected_workspace()` always describe different modes. Make sidebar/preview/dialog logic derive from the repaired selection/runtime helpers instead of inferring mode from corrupted state.

**Step 4: Run tests to verify they pass**

Run:
- `cargo test start_key_opens_dialog_for_main_workspace -- --exact`
- `cargo test stop_key_on_active_main_workspace_stops_agent -- --exact`
- `cargo test merge_key_opens_merge_dialog_for_selected_workspace -- --exact`
- `cargo test update_key_opens_update_from_base_dialog_for_selected_workspace -- --exact`
- `cargo test workspace_pr_token_registers_link_hit_data -- --exact`
- `cargo test waiting_workspace_row_has_no_status_badge_or_input_banner -- --exact`

Expected: PASS

### Task 6: Update command/help metadata and run validation

**Files:**
- Modify: `src/ui/tui/commands/catalog.rs`
- Modify: `src/ui/tui/commands/meta.rs`
- Modify: `src/ui/tui/view/view_overlays_help/keybind_overlay.rs`
- Modify: `src/ui/tui/mod.rs`
- Test: `src/ui/tui/mod.rs`

**Step 1: Write or update the failing test**

Use the existing snapshot/count failures as the acceptance target for command palette and help parity after the model repair.

**Step 2: Run focused tests**

Run:
- `cargo test ui_command_metadata_counts_match_expected_snapshot -- --exact`
- `cargo test keybind_help_mentions_tasks_and_worktrees -- --exact`

Expected: FAIL if discoverability metadata still reflects the pre-refactor behavior.

**Step 3: Write minimal implementation**

Update command metadata and help text so the repaired task/worktree behavior is discoverable and the counts match reality.

**Step 4: Run focused tests to verify they pass**

Run:
- `cargo test ui_command_metadata_counts_match_expected_snapshot -- --exact`
- `cargo test keybind_help_mentions_tasks_and_worktrees -- --exact`

Expected: PASS

**Step 5: Run required local validation**

Run:
- `cargo test ui::state::tests::reducer_moves_selection_with_bounds -- --exact`
- `cargo test ui::tui::tests -- --nocapture`
- `make precommit`

Expected: PASS
