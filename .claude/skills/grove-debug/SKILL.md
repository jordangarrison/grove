---
name: grove-debug
description: >
  Analyze Grove debug snapshots (.grove-debug-snapshot.json) and event logs
  generated with Ctrl+D. Use when debugging visual glitches, flicker,
  preview oscillation, stale output, wrong UI mode, cursor issues, or any
  unexpected TUI behavior in Grove. Trigger phrases: "debug snapshot",
  "analyze debug", "check debug log", "what went wrong", "why is it flickering".
args: "[optional: path to event log .jsonl file]"
allowed-tools: Bash, Read, Glob, Grep, Task
---

# Grove Debug Analyzer

Analyze the Ctrl+D debug snapshot and optional event log to diagnose TUI issues.

## Inputs

- **Snapshot**: `.grove-debug-snapshot.json` in the project root (always present after Ctrl+D)
- **Event log**: NDJSON file passed via `$ARGUMENTS` or found via `grove --event-log <path>`. Optional.

## Step 1: Load snapshot

Read `.grove-debug-snapshot.json` from the project root. If not found, check `$ARGUMENTS` for a path.

Parse and extract:

| Field | What to check |
|---|---|
| `ts` | Convert to human-readable time |
| `mode` | Current UI mode (list, preview, interactive) |
| `focus` | Which pane has focus |
| `viewport` | Terminal dimensions (too small can cause layout bugs) |
| `sidebar_width_pct` | Sidebar ratio |
| `workspace` | Selected workspace, agent type, status, branch |
| `interactive` | If present: cursor position, visibility, pane dimensions, target session |
| `preview` | Line counts, offset, auto_scroll state |
| `last_tmux_error` | Any tmux error at time of snapshot |

## Step 2: Analyze recent captures

The `recent_captures` array (ring buffer, up to 10 entries) is the most valuable diagnostic data. For each capture entry:

- `ts`: timestamp (check intervals between captures for polling frequency)
- `raw_output` / `cleaned_output` / `render_output`: compare across entries
- `changed_raw` / `changed_cleaned`: whether the capture differed from the previous one
- `digest.raw_hash`, `digest.cleaned_hash`, `digest.raw_len`: hash/length for quick diff

### Common patterns to detect

**Flicker/oscillation**: Multiple consecutive captures where `changed_cleaned` alternates true/false with different hashes but similar content. Compare cleaned_output across entries.

**Stale preview**: All recent captures show `changed_raw: false` and `changed_cleaned: false`, but the user reports the terminal has changed. May indicate a polling or capture issue.

**Hash collision**: `changed_cleaned: false` but cleaned_output actually differs (rare, check raw_len changes).

**Render divergence**: `cleaned_output` and `render_output` have different effective content (beyond ANSI codes). Indicates a sanitization bug.

**Capture gaps**: Large timestamp gaps between entries suggest polling stalled or the app was idle.

## Step 3: Check current render state

Compare `current_render_lines` and `current_clean_lines`:

- Are they consistent with the latest capture in `recent_captures`?
- Does `preview.offset` make sense given `preview.line_count`?
- Is `auto_scroll` on when it should be (and vice versa)?

## Step 4: Load event log (if available)

If an event log path was provided via `$ARGUMENTS`, or if the user mentions one, read it.

Event log format is NDJSON with fields: `ts`, `event`, `kind`, `data`.

Key event types:

| event | kind | meaning |
|---|---|---|
| `state_change` | `selection_changed` | Workspace selection changed |
| `state_change` | `focus_changed` | Pane focus switched |
| `mode_change` | `mode_changed` | UI mode transition |
| `mode_change` | `interactive_entered` | Entered interactive mode (data has session name) |
| `mode_change` | `interactive_exited` | Left interactive mode |
| `dialog` | `dialog_opened` | Dialog shown |
| `tmux_cmd` | `execute` | Tmux command ran (data has command string) |
| `error` | `tmux_error` | Tmux error (data has error message) |
| `flash` | `flash_shown` | Flash message displayed |

### Event log analysis

- Look for rapid mode transitions (flicker between modes)
- Check for tmux errors preceding the issue
- Correlate event timestamps with snapshot `ts` to find what happened just before
- Count event frequency to detect loops

## Step 5: Report

Output a structured diagnosis:

```
## Snapshot Summary
- Time: <human readable>
- Mode: <mode> | Focus: <focus>
- Viewport: <w>x<h> | Sidebar: <pct>%
- Workspace: <name> (<agent>, <status>)
- Tmux error: <if any>

## Capture Analysis
- Capture count: <N>
- Time span: <first ts> to <last ts> (<delta>)
- Change pattern: <describe>
- Issue detected: <flicker|stale|none|...>

## Event Log Analysis (if available)
- Events loaded: <N>
- Relevant events near snapshot time: <list>
- Anomalies: <if any>

## Diagnosis
<Root cause hypothesis>

## Suggested Next Steps
<Concrete actions to investigate or fix>
```
