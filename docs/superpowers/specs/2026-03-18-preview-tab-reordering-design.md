# Preview Tab Reordering Design

## Summary

Grove already supports multiple dynamic tabs inside each preview pane, but tab
order is effectively append-only. That makes cleanup awkward once a workspace
accumulates several agent or shell tabs.

The accepted direction is to add direct keyboard reordering for preview tabs:

- `[` and `]` keep switching tabs
- `{` moves the active tab left
- `}` moves the active tab right
- `Home` stays pinned as the leftmost tab
- blocked edge moves do nothing, no wrap

This should feel like a small extension of Grove's existing preview-tab model,
not a new mode.

## Goals

- Let the user reorganize open preview tabs without leaving the keyboard.
- Preserve Grove's current fast tab-switching flow.
- Keep `Home` permanently pinned and non-movable.
- Persist tab order across restart and tmux tab restore.
- Keep help and command-palette discoverability in sync with the new keybinds.

## Non-Goals

- Adding a separate tab-management or reorder mode.
- Allowing `Home` to move or be reordered.
- Adding drag-and-drop in this cut.
- Reordering tabs across workspaces.
- Changing tab creation rules like unique `Git` or adjacent insertion.

## User Decisions

- Reordering happens with bare `{` and `}` while preview is focused.
- Existing `[` and `]` tab switching stays unchanged.
- `Home` remains pinned at the far left.
- Edge moves are no-ops, not wraparound.

## Accepted UX

### Keyboard Behavior

When preview focus is active:

- `[` selects the previous tab
- `]` selects the next tab
- `{` swaps the active non-`Home` tab one position left
- `}` swaps the active non-`Home` tab one position right

The active tab remains active after the move.
Blocked moves are silent no-ops. They should not emit a toast or warning.

### Ordering Rules

- `Home` is always index `0`
- non-`Home` tabs may move relative to one another freely
- moving left from the first movable slot does nothing
- moving right from the last slot does nothing
- `Git` remains unique, but its position among non-`Home` tabs is user-driven

### Discoverability

Update both:

- keybind help content
- command palette actions

Grove already treats these as paired discoverability surfaces. The new actions
must appear in both.

## Architecture

### Keep Identity Separate From Order

The current tab model uses `tab_id` for in-memory identity and tmux restore.
That should remain true.

Do not overload `tab_id` as mutable ordering state. Reassigning ids during
reorder would couple identity to layout and create unnecessary ambiguity around
active-tab tracking, dialog references, and future tab-targeted actions.

Instead, add explicit tab ordering data:

- add a stable `display_order` field to `WorkspaceTab`
- persist that order into tmux metadata with a dedicated key
- restore tabs by `display_order`, with `Home` still forced first

This keeps semantics clear:

- `id` means identity
- `display_order` means visual order

### Reorder Operation

Add a small workspace-tab-state operation that moves the active tab by one slot:

- reject if active tab is `Home`
- reject if requested neighbor does not exist
- reject if requested move would cross the pinned `Home` slot
- swap the two non-`Home` tabs in the vector
- renumber or normalize `display_order` values after the move

The move should be local to `WorkspaceTabsState`, with the app layer handling:

- command dispatch
- metadata rewrite for the full non-`Home` tab set in that workspace
- preview refresh

### Persistence

Today tmux metadata restore reads workspace path, kind, title, agent, and id.
To preserve user-driven order, extend that metadata with order as a first-class
field.

Expected changes:

- write `@grove_tab_order` for every live non-`Home` tab session
- include order in the tmux session listing row
- parse it during restore
- sort restored tabs by `(home first, display_order)`

If order metadata is missing for a restored tab, fall back conservatively to a
stable deterministic order, ideally the current id-based behavior. This keeps
restore robust while the code transitions.

After any reorder, rewrite order metadata for every non-`Home` tab in the
workspace so persisted order stays contiguous and unambiguous.

## Command Model

Add two commands:

- `MoveTabLeft`
- `MoveTabRight`

They should be enabled only when:

- no modal is open
- preview pane is focused
- the selected workspace has an active non-`Home` tab

These commands should no-op cleanly when movement is blocked.

## Testing

Add regression tests that verify behavior, not implementation details.

### Workspace Tab State

Add focused state tests for:

- moving a shell or agent tab left swaps with its left neighbor
- moving a tab right swaps with its right neighbor
- moving the first movable tab left does nothing
- moving the last tab right does nothing
- moving `Home` does nothing
- active tab stays active after reorder

### Keybinding And Command Dispatch

Add tests that verify:

- `{` triggers move-left in preview focus
- `}` triggers move-right in preview focus
- `[` and `]` still switch tabs
- reorder commands are disabled outside preview focus

### Persistence And Restore

Add tests that verify:

- reordered tabs write updated order metadata
- tmux metadata restore reconstructs the stored tab order
- missing order metadata falls back safely

### Discoverability

Add tests or assertions covering:

- command palette entries for move-left and move-right
- help catalog text including the new shortcuts

## Risks

The main risk is conflating tab identity with visual order. That would work for
the immediate feature but make future tab-targeted flows harder to reason
about. Explicit order metadata avoids that.

There is also a restore risk if metadata parsing and writing drift out of sync.
The implementation should update both sides together and cover them with tests.

## Open Questions

None for this cut.
