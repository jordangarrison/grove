# Project Manifest Migration Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make project add create a base-task manifest when the repo is not already represented, and migrate legacy config-only projects into manifests during bootstrap and refresh.

**Architecture:** Keep task discovery manifest-only. Add a small manifest materialization path that can derive a base task from a configured project, skip repos already represented by manifest-backed tasks, and reuse it from project add plus bootstrap/refresh migration.

**Tech Stack:** Rust, existing Grove task lifecycle and TUI tests.

---

## Chunk 1: Manifest Materialization Helper

### Task 1: Add helper and unit tests

**Files:**
- Modify: `src/application/task_lifecycle.rs`
- Test: `src/application/task_lifecycle.rs`

- [ ] **Step 1: Write failing tests**
- [ ] **Step 2: Run targeted tests and verify they fail**
- [ ] **Step 3: Implement helper to materialize a base manifest for one project when missing**
- [ ] **Step 4: Run targeted tests and verify they pass**

## Chunk 2: Bootstrap And Refresh Migration

### Task 2: Materialize legacy projects before discovery

**Files:**
- Modify: `src/ui/tui/bootstrap/bootstrap_discovery.rs`
- Modify: `src/ui/tui/bootstrap/bootstrap_app.rs`
- Modify: `src/ui/tui/update/update_lifecycle_workspace_refresh.rs`
- Test: `src/ui/tui/update/update_lifecycle_workspace_refresh.rs`

- [ ] **Step 1: Write failing refresh/bootstrap migration test**
- [ ] **Step 2: Run targeted test and verify it fails**
- [ ] **Step 3: Implement manifest materialization before discovery**
- [ ] **Step 4: Run targeted test and verify it passes**

## Chunk 3: Project Add Integration

### Task 3: Create manifest on project add

**Files:**
- Modify: `src/ui/tui/dialogs/dialogs_projects_crud.rs`
- Test: `src/ui/tui/mod.rs`

- [ ] **Step 1: Write failing project add integration test**
- [ ] **Step 2: Run targeted test and verify it fails**
- [ ] **Step 3: Implement add flow to create the manifest after config save**
- [ ] **Step 4: Run targeted tests and verify they pass**

## Chunk 4: Validation

### Task 4: Regression validation

**Files:**
- None

- [ ] **Step 1: Run modified targeted tests**
- [ ] **Step 2: Run `make precommit`**
