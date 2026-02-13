---
name: grove-debug
description: >
  Analyze Grove debug record files (.grove/debug-record-*.jsonl), continuous
  NDJSON logs of every frame, state change, input, tmux command, and polling
  event. Use when debugging visual glitches, flicker, input lag, stale preview,
  blank frames, wrong UI mode, cursor issues, or any unexpected TUI behavior.
  Trigger phrases: "debug snapshot", "analyze debug", "check debug log",
  "debug log", "debug record", "look at debug", "check event log",
  "what went wrong", "why is it flickering".
args: "[optional: path to .jsonl debug record file]"
allowed-tools: Bash, Read, Glob, Grep, Task
---

# Grove Debug Record Analyzer

Analyze continuous debug record logs to diagnose TUI issues.

## Inputs

- **Debug record**: `.grove/debug-record-{app_start_ts}-{pid}.jsonl` in the project root (NDJSON, one event per line)
- **Override**: if `$ARGUMENTS` contains a path, use that file instead

## Step 1: Find the debug record file

If `$ARGUMENTS` contains a path, use that. Otherwise, find the most recent `.grove/debug-record-*.jsonl` by modification time:

```bash
ls -t .grove/debug-record-*.jsonl | head -1
```

Print:
- Which file was selected
- File size (`wc -c`)
- Line count (`wc -l`)

## Step 2: Event overview

Count events by `event/kind` to understand session shape:

```bash
jq -r '"\(.event)/\(.kind)"' <file> | sort | uniq -c | sort -rn
```

Show time span:
- First `ts` and last `ts` (convert to human-readable)
- Total duration

Flag anomalies:
- Any `error` events
- `tmux_cmd/completed` with `"ok":false` or missing `ok`
- Large gaps (>5s) between consecutive `tick/scheduled` events
- `frame/rendered` events where `non_empty_line_count` drops to 0 or near-0

## Step 3: Event type reference

| event | kind | frequency | meaning |
|---|---|---|---|
| `debug_record` | `started` | 1 | App startup marker, contains `app_start_ts` |
| `frame` | `rendered` | high | Every frame: full `frame_lines` buffer, `frame_hash`, `mode`, `focus`, `degradation`, `non_empty_line_count`, `seq`, viewport dims, pending input state |
| `frame` | `timing` | high | Frame timing: `draw_ms`, `view_ms`, `frame_log_ms` |
| `tick` | `scheduled` | high | Polling tick scheduled: `interval_ms`, `source` (adaptive), pending depth |
| `tick` | `processed` | high | Polling tick executed: `drained_count`, `early_by_ms`, `late_by_ms` |
| `update_timing` | `message_handled` | high | Message processing: `msg_kind` (key, tick, etc.), `update_ms` |
| `preview_poll` | `capture_completed` | medium | Output capture: `session`, `capture_ms`, `changed`, `output_bytes` |
| `preview_poll` | `cursor_capture_completed` | medium | Cursor poll: `cursor_row`, `cursor_col`, `cursor_visible` |
| `preview_update` | `output_changed` | medium | Preview content changed: `line_count`, `session` |
| `input` | `interactive_key_received` | varies | Key input: `key`, `repeat`, `seq` |
| `input` | `interactive_action_selected` | varies | Key mapped to action: `action`, `seq`, `session` |
| `input` | `interactive_forwarded` | varies | Input forwarded to tmux: `literal_chars`, `tmux_send_ms`, `queue_depth` |
| `input` | `interactive_input_to_preview` | varies | Input visible in preview: `input_to_preview_ms`, `tmux_to_preview_ms`, `consumed_input_count` |
| `input` | `interactive_inputs_coalesced` | varies | Multiple inputs batched: `consumed_input_count`, `consumed_input_seq_last` |
| `tmux_cmd` | `execute` | varies | Tmux command sent: `command` string |
| `tmux_cmd` | `completed` | varies | Tmux command finished: `command`, `duration_ms`, `ok` |
| `state_change` | `selection_changed` | low | Workspace selection: `index`, `workspace` |
| `state_change` | `focus_changed` | low | Pane focus switch: `focus` |
| `mode_change` | `mode_changed` | low | UI mode transition: `mode` |
| `mode_change` | `interactive_entered` | low | Entered interactive mode: `session` |
| `mode_change` | `interactive_exited` | low | Left interactive mode |
| `error` | `tmux_error` | rare | Tmux operation failed |

## Step 4: Frame analysis (key diagnostic)

`frame/rendered` events contain the richest diagnostic data:

- `frame_lines`: the actual rendered TUI (array of strings, one per terminal row)
- `frame_hash`: hash of frame content
- `mode`, `focus`, `degradation`: current UI state
- `non_empty_line_count`: quick blank-frame detector
- `seq`: monotonic frame sequence number
- `output_changing`: whether preview content is actively changing
- `pending_input_depth`, `oldest_pending_input_age_ms`: input queue state

### Analysis steps

1. **Flicker detection**: Compare consecutive `frame_hash` values. Alternating hashes (A, B, A, B) indicate flicker.
2. **Blank frame detection**: Check for `non_empty_line_count` dropping to 0 or near-0.
3. **Degradation**: Check `degradation` field. Values other than `"Full"` indicate frame budget issues.
4. **Visual inspection**: Read `frame_lines` content around the issue timestamp to see exactly what the user saw.
5. **Frame rate**: Check timing between consecutive `frame/rendered` events and corresponding `frame/timing` data (`draw_ms`, `view_ms`).

## Step 5: Input lag analysis

Trace the full input pipeline using `seq` numbers to correlate events:

1. `input/interactive_key_received` (seq=N) -- key arrives
2. `input/interactive_action_selected` (seq=N) -- key mapped to action
3. `input/interactive_forwarded` (seq=N) -- sent to tmux (`tmux_send_ms`)
4. `preview_poll/capture_completed` (next with `changed: true`) -- output captured
5. `input/interactive_input_to_preview` (seq=N) -- latency measured: `input_to_preview_ms`, `tmux_to_preview_ms`

Key metrics:
- `input_to_preview_ms`: total latency from key receipt to visible output change
- `tmux_to_preview_ms`: latency from tmux send to capture
- `queue_depth` in forwarded events: how backed up the input queue is
- `interactive_inputs_coalesced`: indicates batching under load

## Step 6: Common diagnostic patterns

**Flicker**: alternating `frame_hash` on consecutive `frame/rendered` events. Compare the differing `frame_lines` to identify what's oscillating.

**Stale preview**: many consecutive `capture_completed` with `changed: false` while the user expects output to be updating. Check the `session` name and `output_bytes`.

**Input lag**: large `input_to_preview_ms` values (>200ms is noticeable). Check `queue_depth` and whether `interactive_inputs_coalesced` events appear.

**Blank frames**: `frame/rendered` where `non_empty_line_count` drops to 0 or near-0. Check preceding events for errors or mode changes.

**Mode oscillation**: rapid `mode_change` events (multiple within <1s). Check what's triggering the transitions.

**Tmux failures**: `tmux_cmd/completed` with `"ok":false` or `error/tmux_error` events. Check the `command` string for context.

**Polling gaps**: >5s between `tick/scheduled` events indicates the event loop stalled or polling slowed unexpectedly. Check `source` and `interval_ms`.

**Frame budget degradation**: `degradation` != `"Full"` in `frame/rendered` events. Correlate with `frame/timing` to see which phase (`draw_ms`, `view_ms`) is slow.

## Step 7: Report

Output a structured diagnosis:

```
## Debug Record Summary
- File: <path>
- Duration: <first ts> to <last ts> (<delta>)
- Total events: <N>
- Event breakdown: <top event/kind counts>

## Frame Analysis
- Total frames: <N>
- Unique frame hashes: <N>
- Degradation events: <count where != Full>
- Blank frames: <count where non_empty_line_count near 0>
- Flicker detected: <yes/no, with hash pattern if yes>

## Input Analysis (if interactive events present)
- Total key inputs: <N>
- Coalesced batches: <N>
- Median input-to-preview latency: <ms>
- Max input-to-preview latency: <ms>
- Queue depth max: <N>

## Polling & Capture
- Capture count: <N>
- Changed captures: <N> (<pct>%)
- Tmux commands: <N> (failures: <N>)
- Polling interval range: <min>ms to <max>ms

## Anomalies
- <list any errors, gaps, flicker, degradation, blank frames>

## Diagnosis
<Root cause hypothesis based on the data>

## Suggested Next Steps
<Concrete actions to investigate or fix>
```
