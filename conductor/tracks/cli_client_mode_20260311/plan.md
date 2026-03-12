# Implementation Plan: CLI Client Mode

## Phase 1: Remote Evaluation Service
Create a dedicated service to handle communication with a remote qtip server.

- [ ] Task: Write unit tests for `RemoteEvaluator` in `src/tests/remote-evaluator.test.ts`.
- [ ] Task: Implement `RemoteEvaluator` class in `src/services/remote-evaluator.ts`.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Remote Evaluation Service' (Protocol in workflow.md)

## Phase 2: CLI CLI Integration
Update the CLI entry point to handle the `--remote` flag and delegate evaluation.

- [ ] Task: Write unit tests for CLI remote routing in `src/tests/cli-remote.test.ts`.
- [ ] Task: Update `src/cli.ts` to support parsing `--remote <url>`.
- [ ] Task: Refactor `run` function in `src/cli.ts` to use `RemoteEvaluator` when appropriate.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: CLI CLI Integration' (Protocol in workflow.md)

## Phase 3: Documentation and E2E
Finalize documentation and verify end-to-end functionality.

- [ ] Task: Create a comprehensive E2E test scenario with a mock server.
- [ ] Task: Update `docs/getting-started.md` and `README.md` with usage examples for `--remote`.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Documentation and E2E' (Protocol in workflow.md)
