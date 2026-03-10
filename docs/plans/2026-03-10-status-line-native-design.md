# Native StatusLine Chrome Design

**Issue:** `#51`

**Goal:** Replace Grove's custom header/footer chrome layout helper with ftui's native `StatusLine` widget while preserving the current information architecture and a minimal amount of visual structure.

## Constraints

- Prefer native ftui primitives over custom layout code.
- Avoid hacks to preserve full chip-style styling.
- Keep header and footer hit regions unchanged.
- Keep current user-visible content unchanged where practical:
  - header still shows Grove, repo, optional palette indicator
  - footer still shows state, task/worktree context, and `? help` / `Ctrl+K palette`

## Chosen Approach

Use `StatusLine` for both header and footer layout, with plain text `StatusItem::text(...)` labels plus native `StatusItem::key_hint(...)` on the footer right side.

This keeps the migration real:

- alignment comes from ftui, not Grove code
- right-side key hints use a native widget concept
- the remaining styling is only the overall bar background/foreground

## Visual Compromise

The current multi-color chip aesthetic will flatten somewhat. The clean compromise is:

- retain themed header/footer background colors
- retain explicit labeled segments like `Grove`, repo name, `Palette`, `Keys`
- avoid rebuilding the old chip renderer on top of `StatusLine`

## Implementation Notes

- `src/ui/tui/view/view_chrome_header.rs`
  - build a `StatusLine` with left items only
- `src/ui/tui/view/view_status.rs`
  - build a `StatusLine` with left text items and right native key hints
- `src/ui/tui/text/chrome.rs`
  - delete `chrome_bar_line`
  - delete `keybind_hint_spans` if no longer used
- `src/ui/tui/text.rs` and any re-exports/imports
  - remove dead exports/imports

## Testing

Add or update rendering tests to prove:

- header still renders repo identity and optional palette indicator
- footer still renders task/worktree context
- footer still renders both key hints
- hit-region behavior does not regress

## Non-Goals

- full per-segment chip styling parity
- broader theme migration
- changing footer content semantics
