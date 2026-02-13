# ADR-003: Interactive Polling Strategy

**Status:** Accepted  
**Date:** 2026-02-13

## Context

Interactive mode must keep key-to-preview latency low while avoiding runaway
polling cost. Grove forwards input into tmux and renders captured output from
the same session, so polling strategy directly controls perceived typing lag.

## Decision

Use a Sidecar-aligned hybrid polling strategy:

1. Debounced interactive key polls at `20ms`.
2. Adaptive background polling with decay (`50ms`, `200ms`, `500ms`, and
   status-based slower intervals outside active interaction).
3. Earliest-deadline scheduling for ticks, where an already-earlier pending
   tick is retained and never postponed by newer key events.
4. Interactive debounce deadlines are cleared once consumed or on interactive
   exit.

## Details

- Interactive key/paste forwarding schedules a debounced interactive poll
  deadline (`now + 20ms`).
- Global tick scheduling computes the next adaptive deadline and the interactive
  debounce deadline, then selects the earliest.
- If a tick is already pending earlier than the new target, no replacement
  timer is scheduled.
- Tick processing clears consumed interactive debounce deadlines and then polls
  preview output/cursor as normal.

## Consequences

- Continuous typing cannot starve polling by repeatedly pushing deadlines out.
- Fast bursts are still coalesced, but bounded by a short debounce window.
- Adaptive polling behavior remains unchanged for non-interactive paths.
- Event logs can distinguish adaptive scheduling, debounce scheduling, retained
  timers, and processed ticks for deterministic latency debugging.
