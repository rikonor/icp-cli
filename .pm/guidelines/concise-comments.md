---
metadata:
  description: Guideline for writing necessary and clear comments, avoiding redundancy
    and justification.
---

# Guideline: Concise Comments

**Purpose:** To maintain clean and readable code/configuration by avoiding unnecessary comments.

**Guideline:**

1.  **Focus on Clarity:** Comments should primarily explain the _purpose_ or _logic_ of code/configuration that is complex, non-obvious, or has specific external dependencies/constraints.
2.  **Avoid Redundancy:** Do not add comments that merely restate what the code/configuration clearly does (e.g., `# Assign variable x` before `x = 1`).
3.  **Avoid Justification:** Do not add comments explaining _why_ standard syntax was used or _why_ a previous error was corrected (e.g., `# Using 'with' instead of 'secrets' because 'secrets' caused an error`). Assume the correct syntax is intentional.
4.  **Keep Comments Updated:** If code/configuration changes, ensure associated comments are updated or removed to prevent confusion.

**Example (Avoid):**

```yaml
- name: Trigger Distribution Channel Update
  uses: ./.github/workflows/update-distribution-channels.yml
  with: # Using 'with' instead of 'secrets' to avoid parsing error
    HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
```

**Example (Prefer):**

```yaml
- name: Trigger Distribution Channel Update
  uses: ./.github/workflows/update-distribution-channels.yml
  with:
    HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }} # Pass token to reusable workflow
```

_(Note: Even the preferred comment might be optional if the context makes it obvious)_

**Goal:** Keep the codebase clean and focus comments on adding value where understanding might otherwise be difficult.