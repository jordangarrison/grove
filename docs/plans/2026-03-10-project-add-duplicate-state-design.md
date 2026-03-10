# Project Add Duplicate State Design

**Date:** 2026-03-10

## Summary

Improve the Add Project modal so repo matches that are already configured are
visible but clearly unavailable. Avoid the current flow where a duplicate match
looks actionable and only fails after selection with a toast.

## Decision

Keep duplicate repos in the result list, but render them with explicit inline
state:

- show a per-row `Already added` marker
- dim duplicate rows relative to addable rows
- keep them selectable for context, but do not accept them with Enter or mouse
- keep the existing add-time duplicate validation as a final safety check for
  manually entered paths

## UX

- Search results remain a single flat list.
- Addable rows render as today.
- Duplicate rows render with muted path text and an inline `Already added`
  label.
- Results summary and hints mention when some matches are already configured.
- Enter on a selected duplicate row leaves the modal unchanged.
- Clicking a duplicate row only updates selection, it does not accept the row.

## Architecture

- Derive duplicate state from existing configured projects using
  `refer_to_same_location`.
- Store the duplicate bit on `ProjectPathMatch` so render and acceptance share
  the same source of truth.
- Render list rows with ftui rich text (`FtLine`/`FtSpan`) rather than a single
  flat string when duplicate state is present.

## Testing

- Render test for inline `Already added` marker.
- Keyboard regression test ensuring Enter on a duplicate row does not populate
  the path or advance focus.
- Mouse regression test ensuring clicking a duplicate row does not accept it.
