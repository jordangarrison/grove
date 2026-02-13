# ADR-001: FrankenTUI over Ratatui

**Status:** Accepted
**Date:** 2026-02-12

## Context

Grove is a simplified Rust clone of sidecar (a Go/Bubble Tea workspace manager).
The core feature set is workspace management: creating workspaces, assigning AI
agents (Claude Code, Codex) per workspace, and deleting workspaces. We need a
TUI framework that supports list views, text inputs, modal dialogs, split panes,
and tmux integration for agent processes.

The two candidates are FrankenTUI (ftui) and Ratatui, both Rust TUI libraries
with different design philosophies.

## Decision

Use FrankenTUI.

## Rationale

### Built-in Elm runtime

Sidecar is built on Bubble Tea, which uses an Elm-inspired `Model`/`Update`/`View`
architecture. FrankenTUI provides the same pattern natively:

```rust
pub trait Model: Sized {
    type Message: From<Event> + Send + 'static;
    fn init(&mut self) -> Cmd<Self::Message>;
    fn update(&mut self, msg: Self::Message) -> Cmd<Self::Message>;
    fn view(&self, frame: &mut Frame);
    fn subscriptions(&self) -> Vec<Box<dyn Subscription<Self::Message>>>;
}
```

Ratatui is a rendering library only. It provides no runtime, no command system,
no subscription model. We would need to build or import all of that ourselves,
effectively recreating what ftui already ships.

### Terminal lifecycle management

FrankenTUI enforces correct terminal behavior at the kernel level:

- **One-writer rule**: `TerminalWriter` serializes all stdout access.
- **RAII cleanup**: `TerminalSession` restores terminal state even on panic.
- **Sync brackets**: DEC 2026 wraps output for atomic, flicker-free display.

With ratatui, all of this is the application's responsibility.

### Sufficient widget coverage

FrankenTUI provides the widgets this project needs: `List`, `Input`, `Block`,
`Tabs`, `Table`, `Tree`, plus a built-in modal system and pane management. We do
not need a large third-party widget ecosystem.

### Direct architectural mapping

Sidecar's plugin interface (`Init`/`Start`/`Stop`/`Update`/`View`/`Commands`)
maps directly onto ftui's `Model` trait. This makes porting workspace management
concepts (creation modals, agent lifecycle, polling) straightforward rather than
requiring an adapter layer.

## Trade-offs accepted

- **Evolving API**: FrankenTUI is still WIP. We accept the risk of breaking
  changes. The core crates (runtime, render, layout, widgets) are stable enough,
  and the simplified feature set limits our exposure.
- **Smaller ecosystem**: No community crates for extended widgets. If we hit a
  gap, we build it ourselves. Acceptable for a focused workspace manager.
- **Less documentation**: Fewer examples and guides compared to ratatui's mature
  community. Mitigated by having the full source as reference.

## Alternatives considered

### Ratatui

Mature, stable, large ecosystem. Rejected because:

1. No built-in runtime. We would spend significant effort wiring an Elm-style
   architecture, event loop (crossterm), and terminal lifecycle management that
   ftui provides out of the box.
2. The additional ecosystem breadth is unnecessary for our constrained feature
   set.
