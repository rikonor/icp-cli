---
metadata:
  description: Robust shell scripting practices in GitHub Actions
---

# Guideline: Robust Shell Scripting in Actions

**Purpose:** To avoid common pitfalls when using shell commands within GitHub Actions.

**Guideline:**

Avoid passing complex data (e.g., JSON) directly as arguments; use temporary files instead. Beware of fragile shell pipelines (`| head`) that can cause SIGPIPE errors; capture full output first if possible. Use `set -e` or explicit error checks.

**Details:** See [Robust Shell Scripting in GitHub Actions](./docs/ci-cd/robust-shell-scripting.md) for examples and explanations.