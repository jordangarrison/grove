# Migration: 2026-03 Task Model Single-Workspace Import

This document is now a thin pointer.

Use the canonical task-model migration guide:

- [docs/migrations/2026-03-task-model.md](/Users/michael.vessia/projects/grove/docs/migrations/2026-03-task-model.md)

For the old single-workspace migration shape, use that guide with this profile:

- each legacy workspace becomes one task
- each migrated task has exactly one worktree
- do not move or rename existing worktree directories
- write one manifest at `~/.grove/tasks/<task-slug>/.grove/task.toml`
- set both `task.root_path` and `worktrees[0].path` to the existing workspace path

That is the supported import path described in the canonical guide.
