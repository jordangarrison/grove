# PR URL Remote Branch Design

**Date:** 2026-03-09

## Summary

`From GitHub PR` should create the local worktree from the PR's remote head
branch name, not from `pr-<number>`.

## Problem

The current PR create flow fetches the PR head commit correctly, but still
creates the local branch as `pr-<number>`. That loses the real branch identity
from the remote PR and makes the resulting worktree diverge from the branch a
developer expects to resume.

## Decision

For `From GitHub PR`:

1. Parse the PR URL as today.
2. Validate the selected repository matches the PR repository.
3. Resolve the PR `head.ref` from GitHub metadata.
4. Fetch `origin pull/<number>/head`.
5. Reuse or create the local branch named after `head.ref`.
6. Create the worktree from that branch.

The task slug and task root stay `pr-<number>`.

## Branch Reuse Rules

- If the local branch does not exist, create it from `FETCH_HEAD`.
- If the local branch exists and is not checked out in any worktree, move it to
  `FETCH_HEAD`, then add the new worktree from that branch.
- If the local branch exists and is checked out in any worktree, fail and show
  the git error.

## Why This Shape

- It preserves the PR-specific task naming already used in Grove.
- It restores the real remote branch identity inside the worktree and manifest.
- It avoids dangerous force-updates when the branch is active in another
  worktree.

## Metadata Source

Use `gh api repos/<owner>/<repo>/pulls/<number>` to read `head.ref`.

Git alone can fetch the PR head commit, but it does not expose the PR head
branch name. A GitHub metadata lookup is required if Grove wants the real
branch name.

## Error Handling

- If PR metadata lookup fails, task creation fails with a clear message.
- If `git fetch origin pull/<number>/head` fails, task creation fails.
- If branch reuse is blocked because the branch is checked out elsewhere, task
  creation fails.
- Manual task creation remains unchanged.

## Testing

- TUI tests should verify PR mode resolves and passes the remote branch name.
- Task lifecycle tests should verify PR mode creates or reuses the resolved
  branch name.
- Task lifecycle tests should verify an existing checked-out branch fails.
