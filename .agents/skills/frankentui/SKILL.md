---
name: frankentui
description: >
  Look up FrankenTUI framework APIs, patterns, and examples from the reference
  codebase at .reference/frankentui/. Use when asking about FrankenTUI APIs,
  widgets, layout, subscriptions, rendering, styling, hit testing, or any
  ftui-* crate.
  Trigger phrases: "frankentui", "ftui", "how does frankentui", "ftui widget",
  "ftui layout", "ftui subscription", "ftui runtime".
args: "[question or API/concept to look up]"
allowed-tools: Read, Grep, Glob, Task
---

# FrankenTUI Reference Lookup

Answer questions about FrankenTUI by searching the reference codebase.

## Query

$ARGUMENTS

## Reference Location

All FrankenTUI source lives under `.reference/frankentui/`.

## Topic Routing

Based on the query, search **only** the relevant locations. Do not search
everything.

| Query topic | Search first | Then if needed |
|---|---|---|
| Widgets, TextInput, Block, List, Table | `crates/ftui-widgets/src/` | `crates/ftui-extras/src/` |
| Layout, Flex, Constraint, Rect | `crates/ftui-layout/src/` | |
| Styling, Color, Modifier | `crates/ftui-style/src/` | |
| Text, Span, Line | `crates/ftui-text/src/` | |
| Buffer, Frame, rendering, HitGrid, hit testing | `crates/ftui-render/src/` | |
| Runtime, Program, Elm loop, update, view | `crates/ftui-runtime/src/` | |
| Subscriptions, ticks, timers | `crates/ftui-runtime/src/subscription.rs` | `crates/ftui-runtime/src/` |
| Events, keyboard, mouse, input | `crates/ftui-core/src/` | |
| PTY, terminal embedding | `crates/ftui-pty/src/` | |
| Conceptual, architecture, "how does X work" | `docs/` | route to specific crate after |
| "How to do X", usage examples | `crates/ftui-demo-showcase/src/` | `crates/ftui-harness/examples/` |
| Public API surface, re-exports | `crates/ftui/src/lib.rs` | |
| Unknown or broad | Grep across `crates/` for the type/fn name | narrow from results |

All paths above are relative to `.reference/frankentui/`.

## Search approach

1. Identify the topic from the query and pick the matching row above.
2. Search the "first" location. Use Grep for type/function names, Glob for
   file discovery, Read for reading matched files.
3. Only expand to "then if needed" or broader search if the first location
   didn't answer the question.
4. For API questions, find the struct/trait/fn definition and its doc comments.
5. For "how to" questions, find real usage in the showcase or harness before
   reading the definition.

## Output

- Quote the relevant source code or doc sections directly.
- Include file paths and line numbers.
- For "how to" questions, show a concrete example from the reference codebase
  (showcase or harness), not an invented one.
- If no match is found, say so clearly rather than guessing.
