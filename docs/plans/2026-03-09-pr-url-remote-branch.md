# PR URL Remote Branch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `From GitHub PR` create worktrees on the PR remote head branch name, with safe local branch reuse.

**Architecture:** Resolve PR `head.ref` in the TUI create flow, pass it through `TaskBranchSource`, then keep task lifecycle responsible for fetching the PR head commit and reusing or creating the local branch safely before worktree creation.

**Tech Stack:** Rust, git, GitHub CLI (`gh`), FrankenTUI, serde_json

---

### Task 1: Cover PR branch metadata resolution

**Files:**
- Modify: `src/ui/tui/update/update_lifecycle_create.rs`

**Step 1: Write the failing test**

Add a test for PR mode metadata resolution that expects `TaskBranchSource::PullRequest`
to carry both the PR number and the resolved `head.ref`.

**Step 2: Run test to verify it fails**

Run: `cargo test create_dialog_pr_mode_creates_task_with_single_repository`

Expected: FAIL because PR mode only carries the PR number today.

**Step 3: Write minimal implementation**

Add PR metadata lookup and thread the resolved branch name into the create
request.

**Step 4: Run test to verify it passes**

Run: `cargo test create_dialog_pr_mode_creates_task_with_single_repository`

Expected: PASS

### Task 2: Cover PR branch create and safe reuse

**Files:**
- Modify: `src/application/task_lifecycle.rs`
- Modify: `src/application/task_lifecycle/create.rs`

**Step 1: Write the failing tests**

Add tests that verify:
- PR mode creates the local branch from the resolved branch name.
- PR mode moves an existing unused local branch to `FETCH_HEAD`.
- PR mode fails if that branch is checked out in another worktree.

**Step 2: Run tests to verify they fail**

Run: `cargo test create_task_in_root_fetches_pull_request_head_before_worktree_add`

Expected: FAIL because PR mode still uses `pr-<number>`.

**Step 3: Write minimal implementation**

Update task creation to fetch the PR head commit, detect branch existence and
branch occupancy, then create or reuse the branch safely.

**Step 4: Run tests to verify they pass**

Run: `cargo test task_lifecycle`

Expected: PASS for the updated task lifecycle tests.

### Task 3: Keep replay serialization aligned

**Files:**
- Modify: `src/ui/tui/replay/types/completion.rs`

**Step 1: Write the failing test**

Add or update replay request coverage so PR branch source round-trips the
resolved branch name.

**Step 2: Run test to verify it fails**

Run: `cargo test replay`

Expected: FAIL because replay only serializes the PR number today.

**Step 3: Write minimal implementation**

Extend the replay branch-source payload with the PR branch name.

**Step 4: Run test to verify it passes**

Run: `cargo test replay`

Expected: PASS

### Task 4: Validate locally

**Files:**
- Modify: `src/ui/tui/update/update_lifecycle_create.rs`
- Modify: `src/application/task_lifecycle.rs`
- Modify: `src/application/task_lifecycle/create.rs`
- Modify: `src/ui/tui/replay/types/completion.rs`

**Step 1: Run targeted tests**

Run:
- `cargo test create_dialog_pr_mode_creates_task_with_single_repository`
- `cargo test create_task_in_root_fetches_pull_request_head_before_worktree_add`
- `cargo test branch`

Expected: PASS

**Step 2: Run required local validation**

Run: `make precommit`

Expected: PASS
