# Using Composite Actions for Reusability

This document outlines the benefits and process for using composite actions in GitHub Actions workflows to improve maintainability and readability.

## Problem

As workflows grow, you might find identical or very similar sequences of steps repeated across different jobs or even different workflow files. This duplication makes maintenance difficult, as changes need to be applied in multiple places, increasing the risk of errors and inconsistencies.

## Solution: Composite Actions

GitHub Actions allows you to combine multiple workflow steps into a self-contained unit called a **composite action**. This action can then be reused across your workflows just like actions from the Marketplace or other repositories.

**Key Benefits:**

- **Reduces Duplication:** Write the logic once and reuse it multiple times.
- **Improves Readability:** Calling workflows become shorter and easier to understand, as complex logic is hidden within the action.
- **Centralizes Logic:** Makes updates easier, as changes only need to be made in the single composite action definition.
- **Encapsulation:** Clearly defines inputs and outputs, creating a well-defined interface for the reusable logic.

## How to Create and Use

1.  **Create Directory:** Create a directory for your action within your repository, typically under `.github/actions/` (e.g., `.github/actions/my-composite-action/`).
2.  **Define `action.yml`:** Inside the directory, create an `action.yml` file.
    - Define `name` and `description`.
    - Define necessary `inputs` with descriptions and whether they are `required`.
    - Define any `outputs` the action will produce.
    - Specify `runs: using: "composite"`.
    - List the sequence of `steps` that make up the action's logic. These steps can use inputs via the `${{ inputs.input_name }}` syntax and set outputs using standard methods (e.g., `echo "output_name=value" >> $GITHUB_OUTPUT`).
3.  **Use the Action:** In your workflow file(s), replace the sequence of steps with a single `uses:` step pointing to the local path of your composite action:
    ```yaml
    - name: Run My Composite Logic
      uses: ./.github/actions/my-composite-action # Path relative to repo root
      with:
        input_name: some_value
        another_input: ${{ secrets.SOME_SECRET }}
    ```

## Example in this Repository

The `.github/actions/trigger-and-wait-workflow` action is a prime example. It encapsulates three distinct shell steps involving `gh` CLI commands into a single reusable unit, simplifying the `release.yml` and `release-extension.yml` workflows.

By identifying repetitive sequences and extracting them into composite actions, you can significantly improve the structure and maintainability of your CI/CD processes.
