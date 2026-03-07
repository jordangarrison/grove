# Task Reorder Design

## Goal

Remove the deprecated project reorder behavior and replace it with task-scoped ordering that matches Grove's task-first model.

## Problem

The current `Reorder Projects` flow mutates configured repository order in `projects.toml` and then re-sorts discovered workspaces by repository path. That made sense in the old repo-first UI, but it is now wrong:

- the user intent is ordering tasks, not repositories
- reordering repositories can make task rows appear under the wrong group
- the task manifests under `~/.grove/tasks/*/.grove/task.toml` are already the real source of truth

Manual inspection confirmed the manifests are intact. The broken state is config/UI metadata, not on-disk task layout.

## Decision

Adopt task order as the only user-managed ordering concept in the sidebar.

- Remove `Reorder Projects`
- Keep the projects dialog for repository config only
- Load task manifests from `~/.grove/tasks`
- Persist task order separately from project order
- Render and navigate the sidebar in task order

## UX

Use a lightweight in-place reorder mode on the sidebar.

- `Ctrl+R` enters task reorder mode when the list is focused
- The selected task becomes the moving task
- `j/k` and `Up/Down` move the whole task group
- `Enter` saves task order
- `Esc` cancels and restores the previous order

This keeps the affordance similar to the old reorder flow without sending the user into the wrong conceptual model.

## Persistence

Persist task order in Grove's projects-state file alongside `projects` and `attention_acks`.

- field name: `task_order`
- value: ordered list of task slugs

Reasoning:

- task slug is the stable identity already used by manifests and runtime naming
- it is simpler than persisting full root paths
- unknown/new tasks can be appended after persisted entries

## Data Flow

1. On bootstrap and refresh, continue discovering workspaces from configured repositories.
2. Separately load task manifests from `~/.grove/tasks`.
3. Map each discovered workspace path to a task worktree path.
4. Sort `state.workspaces` by:
   - persisted task order
   - manifest worktree order within the task
   - unmatched workspaces last
5. Render the sidebar from task groups, not project groups.

## Reconcile Existing Broken State

Do not move directories on disk.

- task manifests remain canonical
- existing broken project order in `projects.toml` becomes irrelevant to sidebar grouping
- once task ordering ships, the sidebar should reconcile automatically on refresh/startup

## Testing

Add regression coverage for:

- project dialog no longer exposing reorder affordances
- persisted `task_order` round-trip in config
- sidebar grouping/sorting by task manifests instead of project order
- task reorder save/cancel behavior preserving selected workspace
