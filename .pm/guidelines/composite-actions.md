---
metadata:
  description: Using composite actions for workflow reusability
---

# Guideline: Use Composite Actions for Reusability

**Purpose:** To promote cleaner and more maintainable GitHub Actions workflows.

**Guideline:**

Extract repeated sequences of steps into composite actions located in `.github/actions/`. Use the `uses:` keyword with the local path to invoke the action. This improves readability and centralizes logic.

**Details:** See [Using Composite Actions for Reusability](./docs/ci-cd/composite-actions.md) for benefits and creation steps.