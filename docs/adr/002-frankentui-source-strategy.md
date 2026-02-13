# ADR-002: FrankenTUI Dependency Source Strategy

**Status:** Accepted  
**Date:** 2026-02-13

## Context

Phase 0 requires a reproducible FrankenTUI source strategy, with no
developer-local relative paths.

## Decision

Use git dependencies pinned to an exact commit SHA.

- Repository: `https://github.com/Dicklesworthstone/frankentui.git`
- Pin: `507542bea6d84bf1d83c438a2168a0194a7e5264`
- Local path dependencies (for example `../frankentui`) are disallowed.

FrankenTUI crate wiring is deferred to Phase 0.5, but all crate entries must
use this repository + exact commit pin.

## Consequences

- Clean-clone builds remain reproducible across contributors and CI.
- Dependency upgrades are explicit code review events (pin change required).
