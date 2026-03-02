---
name: workspace-scale-benchmark
description: >
  Run and interpret Grove workspace scale performance benchmarks using
  `benchmark-scale`, including baseline capture, regression comparison, and
  optimization loops. Use when user asks about performance, scale, regressions,
  startup/discovery speed, polling/update throughput, p50/p95 timings, or
  benchmark harness usage. Trigger phrases: "benchmark-scale", "workspace scale",
  "perf regression", "p95", "performance baseline", "issue 38".
allowed-tools: Read, Grep, Glob, Task
---

# Workspace Scale Benchmark

Use this skill to measure and improve Grove performance at workspace scale.

## Objective

Produce repeatable performance measurements for:
- discovery
- status-target-generation
- sort-update-pipeline

Then use those measurements to guide and verify optimizations.

## Commands

```bash
# Human-readable benchmark report (N=10,100,500)
cargo run -- benchmark-scale

# JSON report for machine parsing
cargo run -- benchmark-scale --json

# Create or refresh baseline JSON
cargo run -- benchmark-scale --write-baseline docs/workspace-scale-baseline.json

# Compare current run vs baseline (informational warnings)
cargo run -- benchmark-scale --baseline docs/workspace-scale-baseline.json

# Tighten/loosen warning threshold (default 35)
cargo run -- benchmark-scale --baseline docs/workspace-scale-baseline.json --warn-regression-pct 25
```

## Workflow

1. Confirm baseline exists.
- If missing, create one with `--write-baseline`.

2. Collect current benchmark report.
- Run once for quick signal.
- Run 3 times for noisy investigations.

3. Identify dominant flow by p95.
- Prioritize the largest p95 first.

4. Map flow to code area and form one optimization hypothesis.
- `discovery`:
  - `src/infrastructure/adapters/*.rs`
  - `src/application/services/discovery_service.rs`
- `status-target-generation`:
  - `src/application/agent_runtime/polling.rs`
- `sort-update-pipeline`:
  - `src/application/agent_runtime/reconciliation.rs`
  - status/update paths called from polling pipeline

5. Implement one focused change.
- Keep scope small, avoid mixed refactors.

6. Re-run benchmark against baseline.
- Use `--baseline`.
- If warning appears once but not consistently, treat as noise and rerun.

7. Validate quality gates.
- Run `make precommit`.
- Run tests you changed.

8. Report before/after p50 and p95 for affected flow(s).

## Interpretation Rules

- Use p95 for regression decisions.
- Use p50 to gauge typical user experience.
- Single-run spikes are not enough, confirm with repeated runs.
- Baseline warnings are informational only, not hard failures.

## Guardrails

- Do not overwrite baseline unless asked, or when intentionally resetting perf targets.
- If baseline changes intentionally, update both:
  - `docs/workspace-scale-baseline.json`
  - `docs/workspace-scale-benchmarks.md`
- Keep environment notes when sharing numbers (OS, Rust version, run date).

## Handoff Checklist

- Commands run
- Baseline file used
- Flows improved or regressed
- Before/after p50+p95 values
- Any uncertainty due to run-to-run noise
