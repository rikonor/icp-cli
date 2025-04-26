# Workflow Synchronization using `gh` CLI

This document explains a technique for synchronizing GitHub Actions workflows, specifically making a calling workflow (e.g., a release workflow) wait for a triggered workflow (e.g., a distribution update workflow) to complete. This uses the GitHub CLI (`gh`) and offers an alternative to `workflow_call`.

## Problem

When a release tag is pushed, we want to trigger a separate workflow to handle distribution tasks (like updating Homebrew taps or quick-install websites). However, the main release workflow should ideally wait for these distribution tasks to finish successfully before it completes. Using `workflow_call` can achieve this but sometimes introduces structural coupling or complexity that isn't desired if the only requirement is synchronization.

## Solution: `trigger-and-wait-workflow` Composite Action

We implemented a composite action (`.github/actions/trigger-and-wait-workflow`) that encapsulates the necessary `gh` CLI commands.

**Steps performed by the action:**

1.  **Trigger Workflow (`gh workflow run`):**

    - Takes the target workflow filename (e.g., `update-distribution-channels.yml`) and the specific git `ref` (usually the tag name like `v1.2.3` or `my-extension-v0.1.0`) as input.
    - Executes `gh workflow run <workflow_name> --ref <ref_name>`. This sends the trigger event to GitHub Actions.

2.  **Identify Triggered Run (`git rev-parse` & `gh run list`):**

    - The `gh workflow run` command doesn't directly return the ID of the specific run it triggered.
    - The action first uses `git rev-parse <ref_name>` to get the exact commit SHA associated with the tag (`ref`) that triggered the _calling_ workflow (and which the _triggered_ workflow will run against).
    - It waits briefly (`sleep 5`) to allow GitHub Actions time to register the new run.
    - It then executes `gh run list --workflow=<workflow_name> --commit <commit_sha> --limit 1 --json databaseId --jq '.[0].databaseId'`. This command lists runs for the specified workflow, filters them to find the one associated with the correct commit SHA, limits the result to the most recent one, and extracts its unique database ID.

3.  **Wait for Completion (`gh run watch`):**
    - With the unique `RUN_ID` obtained, the action executes `gh run watch $RUN_ID --exit-status`.
    - This command polls the status of the specified workflow run.
    - It waits until the run completes.
    - Crucially, `--exit-status` ensures that this step fails if the watched workflow run fails, thus correctly propagating the failure back to the calling workflow.

## Usage

In the calling workflow (e.g., `release.yml`), replace the individual `gh` commands with:

```yaml
- name: Trigger and Wait for Distribution Update
  uses: ./.github/actions/trigger-and-wait-workflow
  with:
    workflow_name: update-distribution-channels.yml # Target workflow
    ref: ${{ github.ref_name }} # Tag name
    github_token: ${{ github.token }} # Required token
```

## Benefits

- **Decoupling:** Less structural coupling compared to `workflow_call` if only synchronization is needed.
- **Simplicity:** Encapsulates the logic into a reusable action, keeping calling workflows clean.
- **Reliability:** Uses the commit SHA to reliably identify the correct workflow run instance.
- **Failure Propagation:** Ensures failures in the triggered workflow cause the calling workflow to fail.

## Considerations

- Requires `gh` CLI to be available in the runner.
- Requires appropriate `permissions` (e.g., `actions: write` or `actions: read` depending on specific `gh` commands) for the `github_token`.
- The triggered workflow needs access to any required secrets (e.g., `HOMEBREW_TAP_TOKEN`). These are typically inherited if using `gh workflow run` from a runner with access, but need careful consideration if converting the triggered workflow to `workflow_call` later.
