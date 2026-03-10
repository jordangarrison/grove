# Base Task Creation + Upstream Pull & Propagate

## Problem

1. No way to create a "base task" (repo root registration) from the TUI.
   Currently base projects only exist via auto-registration at launch. If a user
   creates a workspace task for a repo without launching Grove from that repo
   first, there's no base worktree entry.
2. No way to pull upstream changes and propagate them to task workspaces in the
   task-based workflow.

## Feature 1: Base Task Creation

### Entry point

Existing "new task" dialog (`n` keybind).

### Flow

Add a task type selection step before the current flow:

- **Workspace** (current behavior): creates a git worktree on a new branch.
- **Base**: registers the repo root as a base task.

When "Base" is selected:

1. Show picker of repos sourced from existing tasks' `repository_path` fields,
   filtered to repos that don't already have a base task
   (`has_base_worktree()` check across all tasks).
2. Create a task where the single worktree entry has `path == repository_path`,
   branch = repo's default branch (detected via git), status = Main.
3. No `git worktree add`, no branch creation. Just register the existing repo
   root.
4. Skip name input. Derive task name from repo directory name (same as
   bootstrap does).

### Domain changes

- New `CreateBaseTaskRequest` (or extend `CreateTaskRequest` with a variant).
- `create_base_task()` in task_lifecycle: validates repo path exists, detects
  default branch, writes task manifest with a single worktree where
  `path == repository_path`.

## Feature 2: Update From Upstream

### Entry point

New keybind/command available when a base worktree is selected.

### Pull step

- Run `git pull --ff-only origin {base_branch}` in the base worktree.
- If fast-forward fails (diverged), show error. User resolves manually in a
  shell tab.

### Propagate step

- After successful pull, scan all tasks for workspaces branched off this repo
  (matching `repository_path` and `base_branch`).
- If any found, prompt: "Update N workspaces from base?"
- On confirm, run existing `update_workspace_from_base` (merge) on each,
  sequentially.
- Show results per workspace: success or failure (merge conflict). User
  resolves conflicts manually.

### Discoverability

- Register keybind in help modal.
- Register command in command palette.

## Out of scope

- Auto-fetch or polling for upstream changes.
- "Behind upstream" indicator (potential follow-up).
- Manual path entry for repos (only known repos from existing tasks).
