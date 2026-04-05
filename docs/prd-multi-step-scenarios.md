# PRD: Multi-Step Scenario Workflows

## Background

### How We Got Here

qtip was designed as a scenario evaluation platform that validates software systems using reusable, declarative YAML definitions. The core insight — the Oracle Pattern — keeps scenarios in a separate, read-only repository so that AI agents cannot cheat by modifying their own tests.

The platform works well for **single-interaction validation**: send one HTTP request or run one CLI command, collect evidence, evaluate checks. This covers a significant class of tests — health checks, API contract validation, CLI output verification, log presence assertions.

However, the systems qtip is meant to validate have grown more complex. Smith, the primary qtip subject, has end-to-end user journeys that span multiple interactions:

1. **GitHub Issue to PR**: Create a loop via CLI, wait for it to complete, verify a PR was created on GitHub, verify labels and comments were applied, clean up test branches.
2. **PRD Multi-Story Consolidation**: Submit a PRD document, wait for N story loops to complete, verify consolidation readiness transitions, wait for the merge loop, verify the final PR.
3. **Guardrail Rejection**: Configure a policy, attempt to trigger a loop, verify it was blocked, verify the rejection comment was posted.
4. **Operator Override**: Create a loop, wait for it to reach running state, cancel it, verify the cancelled state and audit trail.

Each of these requires **ordered steps** where later steps depend on earlier steps' outputs (e.g., a loop ID returned from step 1 is needed in step 2's command). Each step has its own interaction and assertions. Setup and teardown phases bracket the workflow to ensure idempotent, repeatable execution.

qtip's current architecture cannot express these workflows. A `ScenarioFile` has a single `Interaction` and a single `Checks` array evaluated against one `Evidence` object. There is no mechanism for chaining interactions, capturing outputs, or substituting values between steps.

This PRD extends qtip to support multi-step scenario workflows while preserving full backward compatibility with existing single-interaction scenarios.

### Current Architecture (What Exists)

**Scenario model** (`ScenarioFile` in qtip-cli):
- One `Interaction` (type + params)
- One `Vec<Check>` evaluated against one `Evidence`
- `AppliesTo` for capability/interface/environment filtering

**Executor** (`ScenarioExecutor`):
- Dispatches to an `Adapter` based on `interaction.type`
- Adapter returns `Evidence` (status code, JSON body, stdout, stderr, log match)
- `evaluate_checks()` compares checks against evidence, returns failure messages
- Returns `EvaluationResult` (scenario_id, status, evidence, failures)

**Adapter trait**:
- `interaction_type() -> &str`
- `execute(interaction) -> Result<Evidence, String>`
- Three implementations: `CliAdapter`, `ApiAdapter`, `LogAdapter`

**Pipeline** (qtip-cli):
- `load_scenarios(dir)` - parse YAML files
- `resolve_scenarios(manifest, scenarios)` - filter by capabilities/interfaces
- `execute_subject(manifest, scenarios)` - run all, collect results
- Report (text or JSON)

---

## Problem Statement

qtip cannot validate multi-step workflows. Real-world system validation requires sequences of interactions where each step depends on prior steps' results, assertions are made at each step, and setup/teardown phases ensure clean state. Without this capability, qtip is limited to point-in-time checks and cannot serve as the evaluation engine for end-to-end integration testing.

---

## Solution

Extend qtip's scenario schema with `steps`, `setup`, and `teardown` phases. Each step has its own interaction, checks, and optional output capture. Steps execute sequentially with variable substitution from prior step outputs and environment variables. Add new domain-specific check types for common assertion patterns. Preserve full backward compatibility — scenarios without `steps` continue to work exactly as they do today.

---

## User Stories

1. As a scenario author, I want to define an ordered sequence of steps in a scenario YAML file, so that I can model multi-interaction workflows.

2. As a scenario author, I want each step to have its own interaction and checks, so that I can assert on intermediate results, not just the final outcome.

3. As a scenario author, I want to capture values from a step's evidence (e.g., a loop ID from JSON output) and reference them in subsequent steps, so that steps can depend on each other's results.

4. As a scenario author, I want to define setup steps that run before the main workflow, so that I can prepare the environment (reset branches, seed data, configure state).

5. As a scenario author, I want to define teardown steps that run after the main workflow regardless of pass/fail, so that I can clean up test artifacts (delete branches, close PRs).

6. As a scenario author, I want the scenario to fail fast when a main step's checks fail (skip remaining main steps), but still run teardown, so that I get clear failure signals without leaving garbage.

7. As a scenario author, I want to use `$variable` syntax to reference step outputs and environment variables in interaction parameters, so that I can compose dynamic commands.

8. As a scenario author, I want environment variables to take precedence over step outputs for variable resolution, so that externally-configured values (repo URLs, tokens) cannot be accidentally overridden by step outputs.

9. As a scenario author, I want existing single-interaction scenarios to continue working without modification, so that the extension is backward compatible.

10. As a scenario author, I want a `loop_state` check type that asserts on the state field in smith loop JSON output, so that I don't have to write raw JSONPath for the most common assertion.

11. As a scenario author, I want a `github_pr_exists` check type that asserts a PR matching criteria exists in `gh pr list` JSON output, so that I can verify PR creation without manual JSONPath.

12. As a scenario author, I want a `github_labels_match` check type that asserts an issue has specific labels, so that I can verify label side effects.

13. As a scenario author, I want a `github_comment_contains` check type that asserts a comment matching a pattern exists on an issue, so that I can verify comment side effects.

14. As a scenario author, I want the evaluation report to show per-step results (pass/fail, duration, evidence summary), so that I can diagnose which step in a workflow failed and why.

15. As a scenario author, I want to give each step a human-readable `name`, so that reports and error messages are clear.

16. As a scenario author, I want steps to support a `timeout` override, so that long-running steps (like waiting for loop completion) can have different timeouts than quick steps.

17. As a qtip developer, I want the multi-step executor to be a separate code path from the single-step executor, so that the existing single-step path remains simple and untouched.

18. As a qtip user, I want the JSON report format to include step-level detail for multi-step scenarios, so that tooling can parse granular results.

19. As a scenario author, I want to be able to mark a step's checks as `warn_only: true`, so that a check failure logs a warning but does not fail the scenario. This is useful for non-critical assertions like "cost was recorded" that should not block the overall pass/fail.

20. As a scenario author, I want step outputs to support extracting values from both JSON (via JSONPath) and plain text (via regex capture groups), so that I can capture IDs from CLI output that isn't JSON formatted.

---

## Implementation Decisions

### Extended Scenario Schema

A multi-step scenario YAML file:

```yaml
id: SMITH-LOOP-001
name: "GitHub issue to PR delivery"

applies_to:
  capabilities: [github-ingress, loop-lifecycle]
  interfaces: [cli]

acceptance_criteria:
  - id: AC-1
    description: Loop completes successfully
  - id: AC-2
    description: PR is created on the target repository
  - id: AC-3
    description: Test branch is cleaned up

setup:
  - name: "Reset test branch"
    interaction:
      type: cli
      command: "git -C $TEST_REPO checkout main && git -C $TEST_REPO branch -D test-branch || true"

steps:
  - name: "Create loop"
    interaction:
      type: cli
      command: "smith loop create --title 'integration test' --source-type github_issue --source-ref $TEST_REPO_REF --output json"
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
    outputs:
      loop_id:
        from: json
        path: "$.loop_id"

  - name: "Wait for completion"
    timeout: 600
    interaction:
      type: cli
      command: "smith loop wait $loop_id --timeout 10m --output json"
    checks:
      - type: loop_state
        expected: "synced"
        acceptance_criteria: AC-1
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1

  - name: "Verify PR exists"
    interaction:
      type: cli
      command: "gh pr list --repo $TEST_REPO --json number,title,headRefName --state open"
    checks:
      - type: github_pr_exists
        head_ref_pattern: "smith-loop-.*"
        acceptance_criteria: AC-2

teardown:
  - name: "Clean up branches"
    interaction:
      type: cli
      command: "smith-integration cleanup --repo $TEST_REPO --prefix smith-loop"
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-3
```

### Schema Design Choices

**Single `Interaction` vs `steps` detection**: If a scenario has a `steps` array, it is a multi-step scenario. If it has an `interaction` field (no `steps`), it is a single-step scenario. Both cannot be present — this is a parse-time validation error.

**Step outputs**: Each step can declare an `outputs` map. Keys are variable names, values specify extraction:
- `from: json` + `path: "$.foo"` — extract from parsed JSON evidence via JSONPath
- `from: stdout` + `pattern: "ID: (\\w+)"` — extract first regex capture group from stdout
- `from: stderr` + `pattern: "..."` — same, from stderr

Outputs are stored in a `HashMap<String, String>` that accumulates across steps.

**Variable substitution**: Before executing a step's interaction, all `$variable` references in `command`, `path`, `body`, and `headers` are resolved. Resolution order:
1. Environment variables (highest precedence)
2. Step outputs from prior steps
3. Unresolved variables cause a step failure (not silently ignored)

**Execution semantics**:
- Setup steps run sequentially. If a setup step fails, the scenario is aborted (teardown still runs).
- Main steps run sequentially. If a step's checks fail, remaining main steps are skipped (teardown still runs).
- Teardown steps run sequentially and unconditionally. Teardown failures are reported but do not change the scenario's pass/fail status (which is determined by main steps).
- Each step independently dispatches to an adapter based on its `interaction.type`. Different steps can use different adapters (e.g., step 1 uses CLI, step 2 uses API).

**Step timeout**: Optional per-step `timeout` field (seconds). Overrides the adapter's default timeout for that step only. Useful for steps like "wait for loop completion" that may take minutes.

### New Check Types

Four new check types added to the `CheckType` enum:

**`loop_state`** — Parses CLI stdout as JSON, extracts `$.state` (or `$.record.state`), compares to `expected`. Syntactic sugar for a JSONPath check with a well-known path.

**`github_pr_exists`** — Parses CLI stdout as JSON array (from `gh pr list --json`), asserts at least one element matches:
- `head_ref_pattern` (optional): regex match on `headRefName`
- `base_ref` (optional): exact match on `baseRefName`
- `title_pattern` (optional): regex match on `title`
- `state` (optional): exact match on `state`

**`github_labels_match`** — Parses CLI stdout as JSON (from `gh issue view --json labels`), asserts that the `labels` array contains all specified labels:
- `expected`: array of label name strings
- `exact` (optional, default false): if true, the labels must match exactly (no extras)

**`github_comment_contains`** — Parses CLI stdout as JSON array (from `gh issue view --json comments`), asserts at least one comment's `body` matches:
- `pattern`: regex pattern to match against comment body

These check types are evaluated by the same `evaluate_checks` function. They consume `Evidence` (stdout from CLI commands that output JSON). The scenario author is responsible for writing the `gh` CLI command in the interaction; the check type provides structured assertion against the output.

### Crate-Level Changes

**qtip-catalog**:
- Extend `ScenarioFile` deserialization to accept both single-interaction and multi-step formats.
- Add `ScenarioStep` struct: `name`, `interaction`, `checks`, `outputs`, `timeout`, `warn_only`.
- Add `StepOutput` struct: `from` (json/stdout/stderr), `path` (JSONPath), `pattern` (regex).
- Validate mutual exclusivity of `interaction` vs `steps` at parse time.
- New variant `ScenarioKind`: `Single(Interaction, Vec<Check>)` | `Workflow { setup, steps, teardown }`.

**qtip-executor**:
- Add `WorkflowExecutor` alongside existing `ScenarioExecutor`.
- `WorkflowExecutor` takes a `Vec<Adapter>` (same adapters), plus a variable context.
- New method: `execute_workflow(scenario) -> WorkflowResult`.
- `WorkflowResult` contains: `Vec<StepResult>` for setup, main, and teardown phases, plus overall status.
- `StepResult` contains: step name, duration, status (passed/failed/error/skipped/warn), evidence, failures.
- Add new `CheckType` variants: `LoopState`, `GithubPrExists`, `GithubLabelsMatch`, `GithubCommentContains`.
- Implement `evaluate_check` arms for each new type.
- Add `resolve_variables(template: &str, context: &HashMap<String, String>) -> Result<String, String>`.
- Add `extract_outputs(outputs: &HashMap<String, StepOutput>, evidence: &Evidence) -> Result<HashMap<String, String>, String>`.

**qtip-resolver**:
- No changes needed. Resolution operates on `applies_to`, which is identical for both scenario kinds.

**qtip-cli**:
- Detect scenario kind after loading.
- Route single-interaction scenarios to existing `ScenarioExecutor::execute`.
- Route multi-step scenarios to `WorkflowExecutor::execute_workflow`.
- Extend the text reporter to print step-level results for workflow scenarios.
- Extend the JSON reporter to include step-level detail.
- Pipeline functions (`execute_subject`) handle both kinds transparently.

### Report Format Extension

The JSON report for a multi-step scenario:

```json
{
  "scenario_id": "SMITH-LOOP-001",
  "scenario_name": "GitHub issue to PR delivery",
  "kind": "workflow",
  "status": "passed",
  "duration_ms": 45230,
  "setup": [
    { "name": "Reset test branch", "status": "passed", "duration_ms": 120 }
  ],
  "steps": [
    {
      "name": "Create loop",
      "status": "passed",
      "duration_ms": 340,
      "outputs_captured": { "loop_id": "smi-abc123" },
      "checks": [
        { "type": "status_code", "status": "passed" }
      ]
    },
    {
      "name": "Wait for completion",
      "status": "passed",
      "duration_ms": 42000,
      "checks": [
        { "type": "loop_state", "status": "passed", "expected": "synced", "actual": "synced" },
        { "type": "status_code", "status": "passed" }
      ]
    },
    {
      "name": "Verify PR exists",
      "status": "passed",
      "duration_ms": 1200,
      "checks": [
        { "type": "github_pr_exists", "status": "passed" }
      ]
    }
  ],
  "teardown": [
    { "name": "Clean up branches", "status": "passed", "duration_ms": 550 }
  ]
}
```

Single-interaction scenarios retain the existing report format with `"kind": "single"`.

### Variable Resolution Details

The variable resolver is a standalone function in qtip-executor that:
1. Scans the string for `$identifier` patterns (alphanumeric + underscore).
2. Resolves each against the context (env vars first, then step outputs).
3. Returns an error listing all unresolved variables (fail-fast, not partial substitution).
4. Supports escaping: `$$` produces a literal `$`.

The resolver is applied to:
- CLI `command` strings
- API `path`, `body` (recursively into JSON string values), and `headers` values
- Log `query` strings

It is NOT applied to:
- Check fields (`expected`, `path`, `pattern`) — these are static assertions
- Scenario metadata (`id`, `name`, `applies_to`)

---

## Testing Decisions

### What Makes a Good Test

Tests should validate external behavior through qtip's public interfaces. For the executor, this means: given a scenario definition and mock adapter responses, assert on the `WorkflowResult` (step statuses, captured outputs, failure messages). Never assert on internal state like the variable context HashMap — assert on whether the next step received the right command after substitution.

### Modules to Test

**Variable resolver** (`resolve_variables`):
- Substitution from context
- Environment variable precedence
- Unresolved variable error
- Escape handling (`$$`)
- No substitution in empty strings

**Output extractor** (`extract_outputs`):
- JSON path extraction from evidence
- Regex capture from stdout/stderr
- Missing path / no match errors
- Multiple outputs from one step

**Workflow executor** (`execute_workflow`):
- Happy path: setup → steps → teardown all pass
- Step failure: check fails → remaining steps skipped → teardown runs
- Setup failure: setup fails → main steps skipped → teardown runs
- Teardown failure: teardown fails → reported but doesn't change pass/fail
- Output capture and substitution across steps
- Per-step timeout override
- `warn_only` steps: check fails → logged as warning, does not fail scenario

**New check types** (each type):
- Happy path (check passes)
- Failure path (check fails with clear message)
- Malformed evidence (graceful error, not panic)
- Edge cases per type (e.g., `github_labels_match` with `exact: true`)

**Schema parsing** (qtip-catalog):
- Multi-step scenario parses correctly
- Single-interaction scenario still parses (backward compat)
- Both `interaction` and `steps` present → parse error
- Missing required fields in steps → parse error
- Outputs with invalid `from` type → parse error

### Prior Art

- Existing check evaluation tests in `qtip-executor/src/check.rs` (unit tests with mock evidence)
- Existing adapter tests in `qtip-executor/src/adapters/` (mock command execution)
- Existing scenario parsing tests in `qtip-catalog/src/parse.rs`

---

## Out of Scope

- **Parallel step execution** — Steps within a scenario always execute sequentially. Parallelism exists at the scenario level (multiple scenarios can run concurrently), not within a single scenario.
- **Conditional branching** — No `if/else` or `when` conditions on steps. Scenarios are linear sequences. Complex branching logic belongs in the CLI commands within steps, not in qtip's orchestration.
- **Looping / retry within scenarios** — No `repeat` or `until` constructs. If a step needs to poll, the CLI command itself should handle polling (e.g., `smith loop wait` handles its own retry loop).
- **Shared state across scenarios** — Each scenario's variable context is isolated. Scenarios cannot pass outputs to other scenarios.
- **Direct GitHub API integration** — qtip does not call GitHub APIs directly. It evaluates evidence from CLI commands (`gh`, `smith`) that the scenario author writes. This keeps qtip generic and avoids baking in GitHub-specific HTTP logic.
- **Scenario generation** — This PRD covers execution of multi-step scenarios, not AI-assisted authoring. The existing `qtip install skill` flow for Claude-assisted scenario generation is unchanged.
- **Migration tooling** — No automated migration of existing single-step scenarios to multi-step format. They continue to work as-is.

---

## Further Notes

### Backward Compatibility

The single-interaction scenario format is the degenerate case of the multi-step format (one step, no setup, no teardown, no outputs). However, we explicitly keep both code paths rather than converting single-interaction scenarios to single-step workflows internally. This avoids any risk of behavioral change in existing scenarios and keeps the common case (single-interaction) fast and simple.

### Performance Considerations

Multi-step scenarios are inherently sequential and may take minutes (e.g., waiting for a loop to complete). The executor should:
- Stream step-level progress to stderr in verbose mode (so the operator sees "Step 2/4: Waiting for completion..." while it runs).
- Respect per-step timeouts independently.
- Not hold adapter connections open across steps.

### Relationship to Smith Integration Harness

This qtip extension is one component of a larger smith integration testing initiative (see `smith/docs/prds/live-integration-testing.md`). The smith side of that initiative includes `smith loop wait`, a test repo, self-improvement loops, and mise task wiring. This PRD covers only what qtip itself needs to change.

### Incremental Delivery

1. Variable resolver + output extractor (pure functions, fully testable in isolation)
2. `WorkflowExecutor` with setup/steps/teardown lifecycle
3. Schema extension in qtip-catalog (parse multi-step YAML)
4. New check types (independent of multi-step — usable in single-step scenarios too)
5. CLI routing and report format extension
6. Documentation and example scenarios
