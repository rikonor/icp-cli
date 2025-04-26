# Robust Shell Scripting in GitHub Actions

This document provides recommendations for writing more robust and reliable shell scripts within GitHub Actions steps, based on issues encountered and resolved in this repository.

## Common Pitfalls and Solutions

When using `run:` steps with `shell: bash` (or other shells), certain practices can lead to unexpected errors or failures.

### 1. Passing Complex Data as Arguments

**Problem:** Passing multi-line strings, JSON objects, or other complex data directly as command-line arguments to scripts or tools can be unreliable. Shells may interpret quotes, newlines, or special characters in ways that break the argument parsing of the receiving command.

**Solution:**

- **Use Temporary Files:** Write the complex data to a temporary file within the action step. Pass the _path_ to this temporary file as an argument to your script/tool. The script/tool can then read the data directly from the file.
- **Cleanup:** Ensure the temporary file is removed after use, potentially using an `if: always()` condition on the cleanup step to run it even if previous steps failed.

**Example:** Instead of `my_tool --json-data '{ "key": "value with spaces\nand newlines" }'`, use:

```yaml
- name: Run Tool with Complex Data
  shell: bash
  run: |
    echo '{ "key": "value with spaces\nand newlines" }' > temp_data.json
    my_tool --json-path temp_data.json
    rm temp_data.json # Basic cleanup
```

(See `.github/actions/distribution/quick-install/action.yml` for a concrete example using this pattern).

### 2. Fragile Shell Pipelines (SIGPIPE Errors)

**Problem:** Using shell pipelines (`|`) where the reading command (e.g., `head`, `grep`) might exit before the writing command finishes can lead to the writing command receiving a `SIGPIPE` signal and terminating unexpectedly (often with exit code 141). This can happen due to timing or resource issues in the runner environment, even if the pipeline works locally.

**Solution:**

- **Avoid Unnecessary Pipes:** If the goal is just to get the first line or filter output, consider alternatives.
- **Capture Full Output:** Read the entire output of the writing command into a shell variable or array. Process the variable/array using shell built-ins or other tools _after_ the writing command has completed successfully.

**Example:** Instead of `LATEST_TAG=$(gh release list ... | head -n 1)`, use:

```bash
TAGS_OUTPUT=$(gh release list ...) # Capture all output
mapfile -t TAGS_ARRAY <<< "$TAGS_OUTPUT" # Read into array
LATEST_TAG="${TAGS_ARRAY[0]}" # Get first element
```

(See `.github/actions/fetch-latest-extensions/action.yml` for this pattern).

### 3. Explicit Error Checking

**Problem:** By default, shell scripts in actions might continue running even if a command fails, potentially leading to confusing downstream errors.

**Solution:**

- **`set -e`:** Add `set -e` at the beginning of your `run:` script block. This causes the script to exit immediately if any command fails (returns a non-zero exit code).
- **Explicit Checks:** Alternatively, check the exit code of critical commands explicitly:
  ```bash
  if ! my_critical_command; then
    echo "::error::my_critical_command failed!"
    exit 1
  fi
  ```

Adopting these practices can make your shell scripts within GitHub Actions more predictable and easier to debug.
