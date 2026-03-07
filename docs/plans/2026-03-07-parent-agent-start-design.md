# Parent Agent Start UX Design

**Date:** 2026-03-07

## Problem

Grove supports a task-root parent agent session, but after stabilizing `a` to always
mean "new workspace agent tab", there is no explicit way to start the parent
agent from the UI. `Task Home` tells the user that a parent agent belongs there,
but the command surface does not expose a dedicated action.

## Goals

- Add an explicit, discoverable way to start a task-root parent agent.
- Keep `a` stable as "new workspace agent tab".
- Make parent-agent start available from `Task Home`, the command palette, and
  keybind help.
- Reuse the existing launch dialog UX so prompt/init/unsafe settings remain
  configurable.

## Non-Goals

- Reworking task-root session semantics.
- Adding parent-agent tab objects to workspace tab state.
- Changing stop/restart behavior for the existing task-root parent session.
- Supporting multiple concurrent parent agents per task.

## Approved Decisions

### Command model

- `a` remains `New Agent Tab`.
- Add a distinct `Start Parent Agent` command on `A`.
- The new command is only meaningful on `Task Home` for tasks whose root differs
  from the selected workspace path.

### Discoverability

- Add `Start Parent Agent` to the command palette.
- Add `A start parent agent` to keybind help.
- Update the `Task Home` splash copy to distinguish:
  - `A` starts the parent agent in the task root
  - `a` starts workspace agent tabs

### Launch behavior

- `A` opens a task-root-scoped launch dialog.
- The dialog shape stays the same as the existing launch dialog.
- Prompt, init command, and unsafe toggle read/write task-root config markers.
- Launch target:
  - tmux session: `grove-task-<task-slug>`
  - cwd: task root

### Existing-session behavior

- If the parent session already exists, Grove does not create a duplicate.
- Show toast: `parent agent already running`.
- Keep preview focused on `Task Home`.

## UX Flow

### Starting a parent agent

1. User focuses `Task Home`.
2. User presses `A` or chooses `Start Parent Agent` from the palette.
3. Grove opens the launch dialog prefilled from task-root settings.
4. User confirms.
5. Grove launches `grove-task-<task-slug>` in the task root.
6. `Task Home` preview starts showing the parent session output.

### Starting a workspace agent tab

1. User focuses any preview, including `Task Home`.
2. User presses `a`.
3. Grove opens the workspace agent-tab launch dialog.
4. Confirming launches `grove-wt-<task>-<repo>-agent-<n>` in the selected
   workspace path.

This preserves a stable distinction:

- `A` = task-root parent agent
- `a` = selected-workspace agent tab

## Command Availability Rules

`Start Parent Agent` is enabled only when all are true:

- a task is selected
- the selected task has a separate task root
- preview is on `Task Home`
- no start/restart dialog is already in flight

This avoids polluting normal workspace flows and keeps the command tightly
scoped to the place where it makes sense.

## Error Handling

- No selected task: show info toast, do nothing.
- Parent session already exists: show info toast `parent agent already running`.
- tmux launch failure: show the existing launch failure toast path.
- Unsupported/degenerate single-root case: command hidden or disabled.

## Testing Strategy

Add focused regression coverage for:

- `A` from `Task Home` opens a task-root-scoped launch dialog.
- Confirming the dialog launches `grove-task-<task-slug>` with task-root cwd.
- `a` from `Task Home` still launches a workspace agent tab.
- Palette visibility/enabling for `Start Parent Agent`.
- Keybind help and splash-copy updates.
- Duplicate parent session produces the `already running` toast and no second
  launch.

## Rationale

This design keeps the command model coherent instead of making `a`
context-sensitive again. Users get an explicit parent-agent action where they
expect it, while workspace tab creation remains predictable everywhere.
