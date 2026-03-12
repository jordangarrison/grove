# Real Preview Cursor Design

**Date:** 2026-03-11

**Problem**

Grove currently renders the interactive preview cursor by inserting a literal
`|` into preview text. That produces non-terminal behavior, mutates the text
buffer, and creates style-dependent failures. Issue `#63` shows the failure
mode clearly: Claude loses the cursor entirely, while Codex shows a cursor that
still feels wrong.

**Current Behavior**

- Grove disables the frame cursor globally in `src/ui/tui/view/view.rs`.
- The preview pane injects a synthetic cursor marker into rendered preview
  content in `src/ui/tui/view/view_layout.rs`.
- Styled preview content is rendered via parsed terminal spans in
  `src/ui/tui/view/view_preview_content.rs`.
- Tmux cursor capture already provides the actual cursor row, column, and pane
  geometry.

**Decision**

Replace the synthetic preview cursor with the real ftui frame cursor.

Grove should treat preview content and cursor as separate concepts:

- preview content stays untouched
- tmux cursor metadata remains the source of truth
- the preview pane maps tmux cursor coordinates into screen coordinates
- ftui renders the real terminal cursor at that cell

**Why This Approach**

- Matches terminal behavior better than text injection.
- Avoids Claude/Codex divergence caused by ANSI span styling.
- Removes fake cursor artifacts from copied/selected preview text.
- Uses the native ftui cursor path instead of a custom workaround.

**Rejected Alternatives**

1. Keep the fake `|` and adjust styling.

This would reduce one symptom, but the cursor would still not behave like a
real terminal cursor.

2. Render a custom highlighted cell instead of `|`.

Better than text injection, but still a Grove-specific cursor simulation with
its own edge cases.

3. Rebuild the preview as a fully cursor-aware terminal surface.

Possible later, but unnecessary for this fix. We already have tmux cursor
coordinates and ftui frame cursor support.

**Data Flow**

1. Tmux capture updates `session.interactive` cursor state.
2. Preview rendering computes the visible preview range.
3. Preview rendering maps the captured cursor row/col into the current visible
   preview content region.
4. If the cursor is visible and on-screen, Grove sets `frame.set_cursor(...)`
   and `frame.set_cursor_visible(true)`.
5. The terminal renders the native cursor over the existing preview cell.

**Behavior Rules**

- Show the real cursor only for active interactive preview content.
- Do not mutate preview text to show cursor state.
- If the cursor falls outside the visible viewport, do not place it.
- If a modal or text input already owns the cursor, that widget keeps priority.
- Cursor placement should use terminal cell coordinates, not character-count
  insertion logic.

**Expected Code Changes**

- Remove synthetic cursor overlay usage from preview rendering.
- Add a helper that maps interactive cursor state to preview screen
  coordinates.
- Set the real frame cursor from the preview pane render path.
- Replace fake-cursor tests with real frame-cursor assertions.

**Testing Strategy**

- Add regression tests for Claude and Codex interactive preview panes that
  assert:
  - preview text remains unchanged
  - frame cursor position is set correctly
  - frame cursor visibility is enabled
- Add an off-screen cursor test to verify Grove does not place an invalid
  cursor.
- Keep modal/input cursor ownership behavior covered.

**Open Risks**

- Wide-character lines may expose a mismatch between tmux cell columns and any
  remaining char-count-based helper code.
- Cursor placement must be computed relative to preview content rows, not the
  metadata rows at the top of the pane.

**Implementation Direction**

Implement the smallest change that:

- deletes fake preview cursor rendering
- uses the native ftui cursor
- proves the behavior with focused regression tests
