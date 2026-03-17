# Empty State Command Palette Onboarding Design

## Summary

When Grove is empty, the UI should teach the fastest path to a first running
agent.

Instead of passive empty-state copy like "Press 'p' to add a project" or "Press
'n' to create a workspace.", Grove should guide the user to:

- press `Ctrl+K` to open the command palette
- type `help`
- press `p` to add a project
- press `n` to create a task

This is a copy-only onboarding improvement inside the TUI. No new keybind, no
first-run tracking, no separate onboarding mode.

## Goals

- Reduce time-to-first-agent for new users.
- Teach the command palette immediately, not as an advanced feature.
- Keep onboarding inside the TUI where the user already is.
- Reuse existing Grove flows instead of inventing parallel onboarding UI.
- Keep the change small, obvious, and low risk.

## Non-Goals

- Adding a new onboarding modal, coach, or wizard.
- Tracking first launch or onboarding completion.
- Adding a dedicated onboarding keybind.
- Changing task creation, project configuration, or agent launch behavior.
- Replacing keybind help or the command palette.

## User Decisions

- Optimize for fastest first running agent.
- Prefer guidance over automation or machine mutation.
- Keep onboarding inside the TUI.
- Do not add a new `H` keybind.
- Use the empty state to teach `Ctrl+K` and `help`.

## Current Problem

Today Grove's empty-state experience is too passive for a first-time user.

The current copy tells the user isolated facts:

- no projects configured
- press `p` to add a project
- press `n` to create a workspace

That does not teach Grove's core interaction model. A new user still has to
infer that the command palette exists, that help is available there, and that
the path to a running agent is project -> task -> agent.

## Accepted Approach

Update Grove's existing empty-state surfaces so they become action-oriented and
palette-first.

When there are no visible tasks or workspaces, the user should see short,
ordered instructions that introduce the command palette first, then the minimum
next actions needed to get to a running agent.

No new UI mode is introduced. The implementation should only replace existing
empty-state copy in current surfaces.

## User Experience

### Empty-State Message Shape

The message should be short, imperative, and sequenced:

- `Press Ctrl+K for command palette`
- `Type help`
- `Press p to add a project`
- `Press n to create a task`

The exact punctuation may vary slightly to fit the surface, but the order
should stay stable:

1. command palette
2. help
3. project setup
4. task creation

### Why This Order

Leading with `Ctrl+K` teaches Grove's highest-leverage interaction pattern.

Leading with `help` makes the command palette immediately useful, not just a
thing the user has to remember later.

Only after that should the copy point at the minimum setup steps needed before
an agent can run.

## Implementation Surfaces

### Sidebar Empty State

Update the "no projects configured" sidebar empty state in
`src/ui/tui/view/view_chrome_sidebar/render.rs`.

This surface is the first obvious place a new user will look. It should stop at
being instructive, not verbose.

### Preview Or Home Empty State

Update the empty preview or shell copy path that currently renders
`Press 'n' to create a workspace.` in
`src/ui/tui/update/update_navigation_preview.rs`.

This surface should reinforce the same palette-first sequence so the user sees
the same guidance even if they look away from the sidebar.

## Copy Principles

- Keep copy short enough to scan in one glance.
- Prefer imperative verbs: `Press`, `Type`.
- Use Grove's current terms where possible.
- Prefer `task` over `workspace` where the product model has already moved to
  task-first language.
- Avoid multi-paragraph explanation, rationale, or help text in the empty
  state.

## Testing

Add or update regression tests that assert the empty-state behavior rather than
implementation details.

At minimum:

- update existing shell or preview empty-state assertions in `src/ui/tui/mod.rs`
  to expect palette-first guidance
- add a sidebar render assertion for the no-projects case

The tests should verify that empty Grove surfaces now point the user to:

- `Ctrl+K`
- `help`
- `p`
- `n`

## Risks

The main risk is copy drift across surfaces, where one empty state mentions the
palette flow and another still uses older wording. The implementation should
update both primary empty surfaces together.

There is also a terminology risk if one surface still says `workspace` while
the new guidance says `task`. Prefer task-first wording wherever the surrounding
UI already supports it.

## Open Questions

None for this cut. The scope is intentionally constrained.
