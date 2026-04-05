# Workflow Scenarios

This guide documents qtip's workflow scenario format (`setup` + `steps` + `teardown`) and how it executes at runtime.

## Migration Guidance

Existing single-interaction scenarios require no changes.

- Single mode stays: top-level `interaction` + top-level `checks`.
- Workflow mode uses `steps` (with optional `setup` and `teardown`).
- A scenario must choose exactly one mode.

## Schema Reference

### Top-Level Mode Rules

- Single mode: requires `interaction`, requires `checks`, must not define `steps`.
- Workflow mode: requires non-empty `steps`, must not define top-level `interaction` or top-level `checks`.
- Workflow mode may optionally define `setup` and `teardown` arrays.

If both `interaction` and `steps` are present, parsing fails.

### Workflow Step Fields

Each entry in `setup`, `steps`, or `teardown` supports:

- `name` (required, non-empty string)
- `interaction` (required object with `type` and adapter params)
- `checks` (optional array, defaults to `[]`)
- `outputs` (optional map, defaults to `{}`)
- `timeout` (optional positive integer, seconds)
- `warn_only` (optional boolean, defaults to `false`)

### `outputs` Entry Fields

Each output key maps to:

- `from` (required): one of `json`, `stdout`, `stderr`
- `path` (required when `from: json`): JSONPath expression
- `pattern` (required when `from: stdout|stderr`): regex with capture group 1

Captured values are merged into workflow context for later step substitution.

## Variable Substitution

qtip resolves `$variable` templates inside interaction params (including nested JSON fields).

Precedence order:

1. Environment values (process env + scalar manifest environment fields)
2. Previously captured step outputs

Additional behavior:

- Unresolved variables fail step preparation with explicit variable names.
- `$$` escapes a literal `$` (example: `$$HOME` renders as `$HOME`).

## Timeout Behavior

- `timeout` is optional and step-scoped.
- Value must be a positive integer; `0` and negatives are rejected at parse time.
- When set, qtip passes the override only for that step's adapter execution.
- If unset, adapter defaults apply.
- Timeout failures include step name and configured timeout in failure text.

## Execution Lifecycle

- qtip runs workflow phases in order: `setup` -> `steps` -> `teardown`.
- Setup/main are fail-fast for `failed`/`error` statuses; remaining main steps are marked `skipped`.
- Teardown always runs.
- Teardown failures are reported, but main scenario pass/fail is determined by setup+main status.

## Warn-Only Semantics

- `warn_only: true` applies to all checks in that step.
- Check failures are recorded as warnings (`warn`) instead of failing the step.
- Workflow execution does not fail-fast on a `warn` step; later main steps still run.
- Output extraction still runs for `passed` and `warn` steps.
- If output extraction fails, that step becomes `failed`.

## Supported Check Types

- `status_code`
- `json_path`
- `stdout`
- `stderr`
- `log_contains`
- `log_not_contains`
- `loop_state`
- `github_pr_exists`
- `github_labels_match`
- `github_comment_contains`

## Complete Example: GitHub Issue to PR

```yaml
id: SMITH-LOOP-001
name: GitHub issue to PR workflow
applies_to:
  capabilities: [github-ingress, loop-lifecycle]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Loop is created
  - id: AC-2
    description: Loop reaches synced
  - id: AC-3
    description: PR is created from the issue
  - id: AC-4
    description: Cleanup always runs

setup:
  - name: Reset integration branch
    interaction:
      type: cli
      command: smith-integration reset --repo $TEST_REPO
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-4

steps:
  - name: Create GitHub issue
    interaction:
      type: cli
      command: gh issue create --repo $TEST_REPO --title "qtip workflow test" --body "created by qtip" --json number
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
    outputs:
      issue_number:
        from: json
        path: "$.number"

  - name: Create loop from issue
    interaction:
      type: cli
      command: smith loop create --source-type github_issue --source-ref "$TEST_REPO#$issue_number" --output json
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
    outputs:
      loop_id:
        from: json
        path: "$.loop_id"

  - name: Wait for loop completion
    timeout: 600
    interaction:
      type: cli
      command: smith loop wait $loop_id --timeout 10m --output json
    checks:
      - type: loop_state
        expected: synced
        acceptance_criteria: AC-2

  - name: Verify PR exists
    interaction:
      type: cli
      command: gh pr list --repo $TEST_REPO --state open --json number,title,headRefName,baseRefName,state
    checks:
      - type: github_pr_exists
        head_ref_pattern: "smith-loop-.*"
        base_ref: "main"
        state: "OPEN"
        acceptance_criteria: AC-3

teardown:
  - name: Cleanup integration artifacts
    warn_only: true
    interaction:
      type: cli
      command: smith-integration cleanup --repo $TEST_REPO --loop $loop_id --issue $issue_number
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-4
```

The example captures `issue_number` and `loop_id` using `outputs`, then reuses both through `$issue_number` and `$loop_id` in later commands.

## Invalid Example: `interaction` and `steps` Together

```yaml
id: INVALID-MODE-001
name: Invalid mixed-mode scenario
applies_to:
  capabilities: [auth]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Demonstrates schema failure
interaction:
  type: cli
  command: echo "single mode"
steps:
  - name: Also defines workflow step
    interaction:
      type: cli
      command: echo "workflow mode"
```

This fails because qtip enforces mutual exclusivity between single mode and workflow mode. A scenario cannot define both top-level `interaction` and `steps`.
