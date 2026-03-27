# Manual Tab Launch Plan

## Goal

Replace workspace-level auto-start/locking with manual tab launching from a permanent Home tab, and support multiple agent/shell tabs per workspace.

## Non-Goals

1. No backwards compatibility for old workspace-agent coupling.
2. No migration layer for fixed preview tabs.
3. No change to `.grove/base` lifecycle behavior.

## Locked Decisions

1. `a` opens agent picker.
2. `Home` tab is permanent, non-closeable.
3. Git is single-tab per workspace.
4. Rebuild tabs from tmux on startup.
5. Sidebar status shows `N running` (agent tabs only).
6. Remove startup fields from create flow (keep startup config in agent-tab launch flow).
7. `x` kills whatever is running in active tab.
8. `X` closes active tab, confirm kill+close if tab has a live session.
9. Agent picker remembers last selection per workspace.

## Invariants

1. Exactly one `Home` tab exists per workspace.
2. `Home` cannot be closed or killed.
3. Git tab is unique per workspace, `g` always focuses existing tab first.
4. Tab runtime actions (`poll`, `capture`, `interactive`, `stop`) always target active tab session.
5. Sidebar status counts running agent tabs only.

## UX Contract

- Every workspace has a permanent `Home` tab showing the current tree/home content.
- `a` opens picker (Claude/Codex), selected option creates a new agent tab.
- `s` creates a new shell tab.
- `g` opens/focuses the workspace git tab (single instance).
- Repeated `a`/`s` creates additional tabs (adjacent to same-kind tabs).
- `x` stops live session in active tab.
- `X` closes active tab (except `Home`).
- Workspace list is workspace-centric (no assigned workspace agent display).

## Data Model Changes

1. Add tab model:
   - `WorkspaceTabKind = Home | Agent | Shell | Git`
   - `WorkspaceTab`
     - `id`
     - `kind`
     - `title`
     - `session_name` (optional for `Home`)
     - `agent_type` (only for `Agent`)
     - `state` (Running, Stopped, Failed, Starting)
   - `WorkspaceTabsState`
     - `tabs`
     - `active_tab_id`
     - `next_seq`
2. Replace fixed preview-tab state with per-workspace tab-state map.
3. Track `last_agent_selection` per workspace.
4. Remove workspace-level assigned-agent fields and helpers.

## Runtime / Session Changes

1. Session naming supports multiple tab instances:
   - agent: `...-agent-<n>`
   - shell: `...-shell-<n>`
   - git: `...-git` (single)
2. On launch, write tmux metadata (`set-option -t <session> @...`) for restore:
   - workspace path
   - tab kind
   - tab title
   - agent type (agent tabs)
   - tab id
3. On startup/refresh, rebuild tab state from tmux metadata.
4. Polling/interactive/cursor/stop/restart target active tab session, not fixed workspace tab kind.
5. Missing or malformed metadata means ignore session for restore and emit debug log.

## Lifecycle Simplification

1. Remove auto-start behavior after create/refresh:
   - remove pending auto-start agent
   - remove pending auto-launch shell
2. Remove workspace init lock/stamp script and lock dir logic.
3. Run init command directly before process launch in tab session.

## Workspace Decoupling

1. Remove workspace-agent coupling from workspace UI model.
2. Stop reading/writing `.grove/agent` for workspace identity/rendering.
3. Keep `.grove/base` for lifecycle operations.
4. Remove agent selection from create/edit workspace flows.

## Discoverability Updates

1. Add/update command palette actions:
   - New Agent Tab
   - New Shell Tab
   - Open Git Tab
   - Kill Active Tab Session
   - Close Active Tab
2. Update keybind help overlay and footer hints.
3. Update README/docs keybind sections.

## Phased Implementation

1. Add tab model and state migration in memory only.
   - Exit criteria: existing workspace render unchanged, new tab state tests pass.
2. Switch UI to dynamic tabs (render, focus, keyboard nav, mouse hit-testing).
   - Exit criteria: `Home` always visible, active-tab switching works.
3. Add launch flows (`a/s/g`) and last agent selection memory.
   - Exit criteria: repeated `a`/`s` creates adjacent tabs, `g` reuses existing tab.
4. Retarget runtime pipeline to active tab sessions.
   - Exit criteria: poll/capture/interactive/stop/restart act on active tab only.
5. Add tmux metadata write and startup restore.
   - Exit criteria: app restart reconstructs tabs from live tmux sessions.
6. Remove auto-start and workspace init locking.
   - Exit criteria: no pending auto-start state, no lock dir logic, launch still stable.
7. Remove workspace-agent coupling and `.grove/agent` usage.
   - Exit criteria: workspace list/status driven by tab runtime state only.
8. Update discoverability surfaces (palette, keybind modal, footer, docs).
   - Exit criteria: all new actions present in palette and help.
9. Delete obsolete fixed-tab code and tests.
   - Exit criteria: no dead preview-tab paths remain.

## Validation

- Run targeted tests per touched module as each phase lands.
- Run `make precommit` before handoff.

## Test Matrix (Minimum)

1. Tab model
   - home tab presence, non-closeable rules
   - git singleton behavior
   - adjacent insertion rules for agent/shell tabs
2. Launch flows
   - `a` opens picker and uses remembered selection
   - `s` launches shell tab
   - `g` focus existing else create git tab
3. Session controls
   - `x` kills active live session only
   - `X` closes active tab with kill confirm when needed
4. Restore
   - tmux metadata roundtrip
   - malformed metadata ignored safely
5. Workspace list status
   - `N running` counts agent tabs only
