# Performance Modal Design

**Goal:** Expose Grove's own runtime behavior through the existing TUI so users can inspect whether polling, rendering, and session monitoring are behaving efficiently.

## Current State

Grove already has most of the runtime state needed for a useful performance view:

- adaptive poll scheduling in `PollingState`
- frame render logging, including degradation and frame hashes
- per-worktree polling state, output-changing flags, and waiting-prompt tracking
- command palette and keybind help infrastructure
- native ftui modal, block, paragraph, list, and table primitives

What Grove does not currently expose in the UI:

- any dedicated performance or profiling view
- basic Grove process metrics such as CPU and memory
- a user-facing explanation of why Grove is polling at a given cadence

## Constraints

- Use native FrankenTUI primitives already available in the repo.
- Keep the feature read-only.
- Keep the scope narrow, no dashboard or btop-style monitor.
- Show app-level stats plus per-session/worktree detail when data exists.
- Keep command palette and keybind help discoverability in sync.
- Do not add backwards-compatibility shims.
- If process metrics are unavailable, show `unavailable`, do not fake values.

## Investigated UI Shapes

### 1. Dedicated Performance Modal

Open a `Performance` action from `Ctrl+K`, render a centered modal with summary cards and per-session detail.

Pros:

- matches Grove's existing palette-to-modal workflow
- holds both app-wide and per-session information cleanly
- adds no persistent layout overhead

Cons:

- visible only on demand

### 2. Status-Line Snapshot Plus Modal

Keep the detailed modal, but also surface a few headline metrics in the footer status line.

Pros:

- always visible

Cons:

- footer is already crowded
- too little space to explain polling rationale
- increases pressure to over-compress metrics

### 3. Performance Preview Tab

Add a new preview tab beside the existing preview surfaces.

Pros:

- persistent while inspecting a selected target

Cons:

- weaker fit for app-wide scheduler state
- adds permanent tab complexity for a mostly diagnostic feature

## Chosen Approach

Add a dedicated `Performance` modal opened from the command palette.

This modal will show:

- app-level runtime stats
- Grove process CPU and memory
- polling and scheduler rationale
- per-session/worktree rows when data exists

No direct hotkey is added beyond `Ctrl+K`. The new action remains palette-first and appears in keybind help content.

## UI Design

The modal uses native ftui primitives:

- `CommandPalette` for discovery and invocation
- `Modal` for the overlay container
- `Block` to separate sections
- `Paragraph` for short summary and rationale text
- `List` or `Table` for per-session/worktree rows

### Modal Sections

#### Summary

Headline cards or compact blocks for:

- estimated frame cadence or FPS
- recent frame interval summary, average and p95
- next scheduled tick and current trigger/source
- pending input depth and oldest pending input age
- frame degradation state
- Grove process CPU
- Grove process memory

#### Scheduler Rationale

Read-only explanation of why Grove is currently scheduling work at its present cadence, using existing scheduler terms where possible:

- tick source
- tick trigger
- next poll due
- next visual due
- preview poll in flight state
- whether interactive debounce is active
- whether selected output is changing

This section should answer "why is Grove polling right now?" without requiring the user to infer behavior from raw numbers alone.

#### Session Rows

One row per relevant session or worktree context, focused on explanation:

- selected session or preview target
- background polled worktrees
- selected live-preview session when it is excluded from background status polling

Each row should show:

- label
- current status
- effective poll interval or exclusion
- selected vs background role
- output-changing or waiting state when known
- a short reason string describing the current polling decision

## Runtime Data Sources

### Existing Grove State

Use existing TUI/runtime state for:

- `next_tick_due_at`
- `next_tick_interval_ms`
- `next_poll_due_at`
- `next_visual_due_at`
- `interactive_poll_due_at`
- `preview_poll_in_flight`
- pending interactive input depth and age
- selected workspace status
- per-worktree output-changing flags
- per-worktree waiting prompts
- per-worktree idle-poll tracking
- frame degradation

### New App-Owned Performance Snapshot

Add a small app-owned performance snapshot for data not currently stored in UI state:

- recent frame timestamps or intervals for avg/p95 frame timing
- derived FPS estimate
- latest Grove process CPU sample
- latest Grove process memory sample
- timestamp of last successful process-metrics refresh

This snapshot should be a small, bounded structure, not an unbounded event history.

### Process Metrics Sampling

Add one system metrics dependency to sample Grove's own process metrics only.

V1 scope:

- Grove process CPU percent
- Grove process resident memory

Optional if trivial during implementation, but not required:

- process thread count
- total system memory for context

Out of scope:

- full host dashboard
- per-thread breakdown
- historical graphs
- top-like sorting or drill-down

Sampling should be coarse and independent from high-frequency preview polling. The performance view must not meaningfully increase Grove's normal runtime work.

## Command And Help Integration

Add a new `UiCommand::OpenPerformance` with:

- command palette metadata
- enablement rules aligned with other palette-opened modals
- keybind help discoverability text

Because the repo requires command and keybind discoverability to stay in sync, both the palette and help catalog must be updated together.

## Error Handling

- If process metrics cannot be sampled, render `unavailable` for those fields.
- If there are no background polling targets, the session list still renders the selected context if available.
- If frame cadence has insufficient samples, render a clear fallback such as `warming up`.
- Performance data must never block normal rendering or input handling.

## Testing

Add targeted tests for:

- command palette action presence and execution
- keybind help discoverability for the new performance action
- modal rendering with expected section labels and representative values
- polling-reason labeling for selected, background, and excluded-session cases
- process-metrics formatting and unavailable-state behavior
- frame cadence summary behavior with small sample windows

Validation before handoff:

- focused tests added or modified for this feature
- `make precommit`

## Non-Goals

- building a full profiler
- rendering graphs or time-series charts
- exposing every internal counter Grove has
- adding a dedicated hotkey outside the palette
- adding session controls to the performance modal
- reworking Grove's polling policy as part of this feature
