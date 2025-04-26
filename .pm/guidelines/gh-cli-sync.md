---
metadata:
  description: Using gh CLI for workflow synchronization
---

# Guideline: Workflow Synchronization via `gh` CLI

**Purpose:** To recommend a method for triggering and synchronizing dependent GitHub Actions workflows.

**Guideline:**

Use the `.github/actions/trigger-and-wait-workflow` composite action to trigger another workflow and wait for its completion. This uses `gh` CLI commands (`workflow run`, `run list --commit`, `run watch`) for synchronization.

**Details:** See [Workflow Synchronization using `gh` CLI](./docs/ci-cd/gh-cli-sync.md) for implementation details and rationale.