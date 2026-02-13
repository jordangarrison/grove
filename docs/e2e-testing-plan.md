# E2E Testing Plan

Strategies for autonomous, agent-driven testing of Grove. The goal: an agent
can make a code change, run tests, get parseable pass/fail feedback, and fix
failures without manual involvement.

## Problem

Grove is a TUI app that renders to a terminal via tmux. Current testing:

- **Unit tests**: Extensive, cover model/update logic with `RecordingTmuxInput`
- **Flicker script**: `scripts/check-codex-flicker.sh`, narrow (only detects
  ANSI oscillation), depends on nested tmux which is fragile

What's missing: structured feedback loops that let an agent verify both
behavior and rendering after code changes.

---

## Strategy 1: Structured Event Log (`--event-log`)

### Concept

Add a runtime flag that writes newline-delimited JSON (NDJSON) to a file,
emitting one event per state transition. Tests (or agents) read the log to
assert on behavior without parsing terminal output.

### Event Schema

```json
{"ts":1707840000000,"event":"state_change","kind":"selection_changed","data":{"index":1,"workspace":"feature-auth"}}
{"ts":1707840000050,"event":"mode_change","kind":"interactive_entered","data":{"session":"grove-ws-feature-auth"}}
{"ts":1707840000100,"event":"tmux_cmd","kind":"send_keys","data":{"keys":"ls\n","target":"grove-ws-feature-auth"}}
{"ts":1707840000150,"event":"preview_update","kind":"output_changed","data":{"line_count":42,"digest":"a1b2c3"}}
{"ts":1707840000200,"event":"error","kind":"tmux_error","data":{"message":"session not found"}}
```

### Events to Emit

| Category | Events |
|----------|--------|
| Navigation | `selection_changed`, `focus_changed`, `mode_changed` |
| Interactive | `interactive_entered`, `interactive_exited`, `key_forwarded` |
| Agent lifecycle | `agent_started`, `agent_stopped`, `agent_status_changed` |
| Dialogs | `dialog_opened`, `dialog_confirmed`, `dialog_cancelled` |
| Preview | `output_changed`, `scrolled`, `autoscroll_toggled` |
| Errors | `tmux_error`, `discovery_error`, `validation_error` |
| Flash | `flash_shown` (includes message text for assertion) |

### Implementation

1. **CLI flag**: Add `--event-log <path>` to `main.rs` arg parsing. Optional,
   no-op when absent.

2. **EventLogger trait + impl**:
   ```rust
   pub trait EventLogger: Send {
       fn log(&self, event: Event);
   }

   pub struct FileEventLogger {
       writer: Mutex<BufWriter<File>>,
   }

   pub struct NullEventLogger; // no-op default
   ```

3. **Wire into GroveApp**: Add `event_log: Box<dyn EventLogger>` field.
   Instrument `update()` at key transition points. The logger is append-only,
   no reads during runtime.

4. **Emit points** (changes to `src/tui.rs` update function):
   - After `reduce()` calls that change selection/focus/mode
   - On dialog open/confirm/cancel branches
   - On interactive enter/exit
   - On `poll_preview()` when output digest changes
   - On tmux execute calls (wrap or instrument `TmuxInput`)
   - On flash message creation

5. **Test utility**: `EventLogReader` that tails the NDJSON file, supports
   `wait_for(event_kind, timeout)` and `assert_sequence(vec![...])`.

### File Changes

| File | Change |
|------|--------|
| `src/main.rs` | Parse `--event-log <path>` arg |
| `src/event_log.rs` | New: `Event`, `EventLogger`, `FileEventLogger`, `NullEventLogger` |
| `src/tui.rs` | Add `event_log` field to `GroveApp`, emit at transition points |
| `src/lib.rs` | Export `event_log` module |
| `tests/support/mod.rs` | Add `EventLogReader` with `wait_for` / `assert_sequence` |

### Example Test (integration)

```rust
#[test]
fn start_agent_emits_lifecycle_events() {
    let log_path = tempdir().path().join("events.jsonl");
    let mut app = fixture_app_with_event_log(&log_path);

    // Drive through start flow
    app.update(Msg::Key(key('s')));           // open dialog
    app.update(Msg::Key(key(KeyCode::Enter))); // confirm

    let reader = EventLogReader::open(&log_path);
    reader.assert_sequence(&[
        ("dialog_opened", json!({"kind": "launch"})),
        ("dialog_confirmed", json!({"kind": "launch"})),
        ("agent_started", json!({"workspace": "feature-auth"})),
    ]);
}
```

### Agent Feedback Loop

An agent fixing a bug would:
1. Make code change
2. Run `cargo test` (unit tests catch logic regressions)
3. Run an integration test that launches grove with `--event-log`, sends
   scripted key events through the update function, reads the event log
4. Parse structured JSON pass/fail, fix and retry

---

## Strategy 3: Model-Layer Property Testing

### Concept

Since `update()` is `(&mut GroveApp, Msg) -> Cmd<Msg>`, we can generate
random message sequences and assert that invariants always hold. This catches
edge cases no human would write explicit tests for.

### Invariants to Enforce

| ID | Invariant | Rationale |
|----|-----------|-----------|
| P1 | `selected_index < workspaces.len() \|\| workspaces.is_empty()` | Selection never out of bounds |
| P2 | `preview.offset <= preview.lines.len()` | Scroll never past content |
| P3 | At most one of `{launch_dialog, create_dialog, interactive}` is `Some` | Modal exclusivity |
| P4 | `interactive.is_some()` implies selected workspace status is `Active` | Can't interact with stopped agent |
| P5 | Main workspace status is always `Main` | Domain invariant from `Workspace::try_new` |
| P6 | `sidebar_width_pct` is within `[10, 90]` | Divider clamp bounds |
| P7 | No panic on any message sequence | Robustness baseline |

### Implementation

1. **Add `proptest` dev-dependency** to `Cargo.toml`.

2. **Create `Arbitrary` for `Msg`**:
   ```rust
   // src/tui.rs #[cfg(test)]
   use proptest::prelude::*;

   fn arb_key_event() -> impl Strategy<Value = KeyEvent> {
       prop_oneof![
           Just(key('j')),
           Just(key('k')),
           Just(key('s')),
           Just(key('x')),
           Just(key('n')),
           Just(key('!')),
           Just(key('q')),
           Just(key('G')),
           Just(key(KeyCode::Tab)),
           Just(key(KeyCode::Enter)),
           Just(key(KeyCode::Esc)),
           Just(key(KeyCode::Up)),
           Just(key(KeyCode::Down)),
           Just(key(KeyCode::PageUp)),
           Just(key(KeyCode::PageDown)),
           // printable chars for dialog input
           ('a'..='z').prop_map(key),
       ]
   }

   fn arb_msg() -> impl Strategy<Value = Msg> {
       prop_oneof![
           arb_key_event().prop_map(Msg::Key),
           Just(Msg::Tick),
           Just(Msg::Noop),
           (1u16..200, 1u16..60).prop_map(|(w, h)| Msg::Resize {
               width: w, height: h,
           }),
       ]
   }
   ```

3. **Property test harness**:
   ```rust
   proptest! {
       #[test]
       fn no_panic_on_random_messages(
           msgs in prop::collection::vec(arb_msg(), 1..200)
       ) {
           let mut app = fixture_app();
           for msg in msgs {
               let _ = app.update(msg);
           }
       }

       #[test]
       fn selection_always_in_bounds(
           msgs in prop::collection::vec(arb_msg(), 1..200)
       ) {
           let mut app = fixture_app();
           for msg in msgs {
               let _ = app.update(msg);
               if !app.state.workspaces.is_empty() {
                   prop_assert!(
                       app.state.selected_index < app.state.workspaces.len()
                   );
               }
           }
       }

       #[test]
       fn modal_exclusivity(
           msgs in prop::collection::vec(arb_msg(), 1..200)
       ) {
           let mut app = fixture_app();
           for msg in msgs {
               let _ = app.update(msg);
               let active_modals = [
                   app.launch_dialog.is_some(),
                   app.create_dialog.is_some(),
                   app.interactive.is_some(),
               ].iter().filter(|&&b| b).count();
               prop_assert!(active_modals <= 1);
           }
       }

       #[test]
       fn scroll_offset_in_bounds(
           msgs in prop::collection::vec(arb_msg(), 1..200)
       ) {
           let mut app = fixture_app();
           for msg in msgs {
               let _ = app.update(msg);
               prop_assert!(
                   app.preview.offset <= app.preview.lines.len()
               );
           }
       }
   }
   ```

4. **Shrinking**: `proptest` automatically shrinks failing sequences to the
   minimal reproduction. An agent gets a tiny, readable failing case instead
   of a 200-message wall.

### File Changes

| File | Change |
|------|--------|
| `Cargo.toml` | Add `proptest` to `[dev-dependencies]` |
| `src/tui.rs` (test module) | Add `arb_msg()`, `arb_key_event()`, property tests |

### Agent Feedback Loop

Property tests run as normal `cargo test`. Failures produce:
```
thread 'tui::tests::selection_always_in_bounds' panicked
  minimal failing input: msgs = [Key('s'), Key(Enter), Key('j'), Key('j'), Key('j')]
```

This is directly actionable by an agent: replay the 5-message sequence,
inspect the state, find the off-by-one.

---

## Strategy 4: Buffer-Based Render Assertions

### Concept

FrankenTUI's `Frame` and `Buffer` types let us render the `view()` function
into an in-memory grid, then inspect individual cells (character, foreground
color, background color, style attributes). No terminal needed.

### FrankenTUI Test API

```rust
use ftui::render::{Frame, Buffer, Cell};
use ftui::render::GraphemePool;

// Create render target
let mut pool = GraphemePool::new();
let mut frame = Frame::new(80, 24, &mut pool);

// Render the app's view
app.view(&mut frame);

// Inspect cells
let cell: &Cell = frame.buffer.get(x, y).unwrap();
let ch: Option<char> = cell.content.as_char();
let fg: PackedRgba = cell.fg;
let is_bold: bool = cell.attrs.flags().contains(StyleFlags::BOLD);
```

### Test Utilities to Build

```rust
// tests/support/render.rs

/// Extract text content from a row range
fn row_text(frame: &Frame, y: u16, x_start: u16, x_end: u16) -> String {
    (x_start..x_end)
        .filter_map(|x| frame.buffer.get(x, y)?.content.as_char())
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// Find the first row containing the given substring
fn find_row_containing(frame: &Frame, text: &str, width: u16) -> Option<u16> {
    (0..frame.height()).find(|&y| row_text(frame, y, 0, width).contains(text))
}

/// Assert a cell has specific style attributes
fn assert_cell_style(frame: &Frame, x: u16, y: u16, flags: StyleFlags) {
    let cell = frame.buffer.get(x, y).expect("cell in bounds");
    assert!(
        cell.attrs.flags().contains(flags),
        "cell ({x},{y}) expected {flags:?}, got {:?}",
        cell.attrs.flags()
    );
}

/// Assert a row contains text with a specific foreground color
fn assert_row_fg(frame: &Frame, y: u16, x_range: Range<u16>, expected_fg: PackedRgba) {
    for x in x_range {
        if let Some(cell) = frame.buffer.get(x, y) {
            if cell.content.as_char().map_or(false, |c| !c.is_whitespace()) {
                assert_eq!(cell.fg, expected_fg, "cell ({x},{y}) wrong fg color");
            }
        }
    }
}
```

### Example Tests

```rust
#[test]
fn sidebar_shows_workspace_names() {
    let mut app = fixture_app_with_workspaces(vec!["auth", "payments", "docs"]);
    let mut pool = GraphemePool::new();
    let mut frame = Frame::new(80, 24, &mut pool);

    app.view(&mut frame);

    // Workspace names appear in the sidebar region
    assert!(find_row_containing(&frame, "auth", 30).is_some());
    assert!(find_row_containing(&frame, "payments", 30).is_some());
    assert!(find_row_containing(&frame, "docs", 30).is_some());
}

#[test]
fn selected_workspace_is_highlighted() {
    let mut app = fixture_app_with_workspaces(vec!["alpha", "beta"]);
    app.state.selected_index = 1;

    let mut pool = GraphemePool::new();
    let mut frame = Frame::new(80, 24, &mut pool);
    app.view(&mut frame);

    let beta_row = find_row_containing(&frame, "beta", 30).unwrap();
    assert_cell_style(&frame, 2, beta_row, StyleFlags::BOLD);
}

#[test]
fn modal_dialog_renders_over_sidebar() {
    let mut app = fixture_app();
    app.launch_dialog = Some(LaunchDialogState::default());

    let mut pool = GraphemePool::new();
    let mut frame = Frame::new(80, 24, &mut pool);
    app.view(&mut frame);

    // Dialog title visible
    assert!(find_row_containing(&frame, "Start Agent", 80).is_some());
}

#[test]
fn status_bar_shows_flash_message() {
    let mut app = fixture_app();
    app.flash = Some(FlashMessage::new("Agent started"));

    let mut pool = GraphemePool::new();
    let mut frame = Frame::new(80, 24, &mut pool);
    app.view(&mut frame);

    let status_row = frame.height() - 1;
    let text = row_text(&frame, status_row, 0, 80);
    assert!(text.contains("Agent started"));
}

#[test]
fn preview_pane_renders_ansi_colors() {
    let mut app = fixture_app();
    app.preview.render_lines = vec![
        "\x1b[32mSuccess\x1b[0m: all tests passed".to_string(),
    ];

    let mut pool = GraphemePool::new();
    let mut frame = Frame::new(80, 24, &mut pool);
    app.view(&mut frame);

    // Find "Success" text in preview area, verify green foreground
    let preview_x_start = 32; // after sidebar + divider
    if let Some(row) = find_row_containing(&frame, "Success", 80) {
        // Check that the 'S' in "Success" has green fg
        let s_col = (preview_x_start..80)
            .find(|&x| frame.buffer.get(x, row)
                .and_then(|c| c.content.as_char()) == Some('S'))
            .unwrap();
        let cell = frame.buffer.get(s_col, row).unwrap();
        assert_eq!(cell.fg.g(), 255); // green channel high
    }
}
```

### Combining with Property Testing

```rust
proptest! {
    #[test]
    fn view_never_panics(
        msgs in prop::collection::vec(arb_msg(), 0..100),
        width in 20u16..200,
        height in 5u16..60,
    ) {
        let mut app = fixture_app();
        for msg in msgs {
            let _ = app.update(msg);
        }
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(width, height, &mut pool);
        app.view(&mut frame); // must not panic at any size
    }

    #[test]
    fn view_fills_status_bar_row(
        msgs in prop::collection::vec(arb_msg(), 0..50),
    ) {
        let mut app = fixture_app();
        for msg in msgs {
            let _ = app.update(msg);
        }
        let mut pool = GraphemePool::new();
        let mut frame = Frame::new(80, 24, &mut pool);
        app.view(&mut frame);

        // Last row should have some content (status bar always renders)
        let status = row_text(&frame, 23, 0, 80);
        prop_assert!(!status.is_empty(), "status bar should not be blank");
    }
}
```

### File Changes

| File | Change |
|------|--------|
| `tests/support/mod.rs` | Add `render` submodule with `row_text`, `find_row_containing`, assertion helpers |
| `src/tui.rs` (test module) | Add render-based view tests, proptest rendering invariants |

### Agent Feedback Loop

Buffer tests run as `cargo test`. Failures are concrete:
```
assertion failed: cell (5,3) expected BOLD, got (empty)
```
or
```
assertion failed: find_row_containing(&frame, "Start Agent", 80).is_some()
```

An agent can directly map these to the `view()` function and fix rendering
logic.

---

## Implementation Order

### Phase A: Property Testing (Strategy 3)

Start here. Lowest implementation cost, highest bug-finding potential.

1. Add `proptest` to dev-dependencies
2. Implement `arb_msg()` and `arb_key_event()` strategies
3. Add the four core property tests (no-panic, selection bounds, modal
   exclusivity, scroll bounds)
4. Run, fix any discovered violations
5. Add to CI (`make test` already runs `cargo test`)

### Phase B: Buffer Render Assertions (Strategy 4)

Second priority. Catches rendering regressions that property tests miss.

1. Create `tests/support/render.rs` with helper functions
2. Add basic view tests (sidebar names, selected highlight, status bar)
3. Add modal overlay render tests
4. Add proptest rendering invariants (`view_never_panics`,
   `status_bar_not_blank`)
5. Add ANSI color fidelity assertions for preview pane

### Phase C: Event Log (Strategy 1)

Last. Most implementation surface area, but enables the richest feedback
loops for complex scenarios.

1. Create `src/event_log.rs` with trait + file + null implementations
2. Add `--event-log` CLI flag
3. Instrument `update()` with emit points (start with 5-6 key events)
4. Create `EventLogReader` test utility
5. Write first integration test using event log assertions
6. Expand event coverage incrementally as needed

---

## What This Replaces

Once these strategies are in place, the flicker script
(`scripts/check-codex-flicker.sh`) becomes redundant: the property tests
catch state-level flicker causes, the buffer tests catch render-level
issues, and the event log catches behavioral regressions. The script can be
retired or kept as a supplementary real-tmux smoke test.

## Non-Goals

- **Screenshot/pixel diffing**: Overkill for a text-mode TUI, buffer
  inspection is sufficient.
- **Nested tmux harness**: Fragile, hard to debug, not worth generalizing
  beyond the existing flicker script.
- **Browser-based testing**: Not applicable.
- **Record/replay**: Brittle for a rapidly evolving UI, better to assert on
  structured output.
