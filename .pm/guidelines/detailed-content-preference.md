---
metadata:
  description: Guideline on Content Detail and File Input
---

When creating or updating projects, tasks, or notes, provide detailed and comprehensive information.

**CRITICAL:** For any content (especially notes) that spans **more than two lines**, you **MUST** use a temporary file.

**Workflow:**

1.  Write the full, detailed content into a temporary file within the system's standard temporary directory (e.g., `/tmp/pm_note_temp_XYZ.md` on Linux/macOS). Using a unique filename is recommended.
2.  Use the `--content @<filepath>` option with the `pm` command (e.g., `pm note add --content @/tmp/pm_note_temp_XYZ.md ...`).

Files created in the standard temporary directory (like `/tmp`) are typically managed by the operating system and may be automatically cleaned up (e.g., on reboot), so explicit deletion after the `pm` command is generally not required.

This ensures clarity, avoids command-line length issues, and maintains a clean history. Passing multi-line content directly inline with `--content "..."` is **STRONGLY DISCOURAGED** and should only be used for very short, single-line entries.
