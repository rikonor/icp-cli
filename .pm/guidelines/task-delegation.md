---
metadata:
  description: Instructions for proper task delegation
---

## Task Delegation

To manage complexity or prevent excessive context growth (e.g., from repetitive commands with large outputs like testing loops), Roodiger may propose delegating specific, well-defined subtasks.

- **Mechanism:** Delegation uses the `new_task` tool to create a new, focused session.
- **Target Mode:** By default, delegation targets another instance of the **Roodiger** mode (`architect-unlock`) to maintain our collaborative model while isolating the subtask.
- **Trigger:** Delegation might be proposed during the planning phase (Step 3) if a task segment is identified as complex, potentially context-heavy, or logically separable. User agreement is required before delegating.
- **Instructions (`new_task` message):** When delegating, the `message` parameter must include:
  1.  **Context:** Sufficient background from the parent task.
  2.  **Scope:** A precise definition of the subtask's objective and boundaries.
  3.  **Focus:** An explicit statement that this is a delegated subtask with a limited scope, overriding general exploratory behaviors. The subtask should execute the request directly.
  4.  **Completion/Failure Reporting:** Clear instruction to use the `attempt_completion` tool upon finishing.
      - On success: The `<result>` should summarize the work done and outcome.
      - On failure/blocker: The `<result>` must state the inability to complete, describe the issue, and provide relevant context/logs.
  5.  **Superseding Clause:** A note that these specific instructions take precedence over standard mode guidelines for this subtask.
- **Parent Task Role:** The parent Roodiger instance remains active, tracks the subtask's progress, analyzes the result (success or failure) upon completion, and integrates it back into the main workflow.
