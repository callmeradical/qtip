# Implementation Plan: CLI Client Mode

## Phase 1: Remote Evaluation Service [checkpoint: b9e1d87]
Create a dedicated service to handle communication with a remote qtip server.

- [x] Task: Write unit tests for `RemoteEvaluator` in `src/tests/remote-evaluator.test.ts`. 584f259
- [x] Task: Implement `RemoteEvaluator` class in `src/services/remote-evaluator.ts`. 584f259
- [x] Task: Conductor - User Manual Verification 'Phase 1: Remote Evaluation Service' (Protocol in workflow.md) b9e1d87

## Phase 2: CLI CLI Integration
Update the CLI entry point to handle the `--remote` flag and delegate evaluation.

- [x] Task: Write unit tests for CLI remote routing in `src/tests/cli-remote.test.ts`. 33c0066
- [x] Task: Update `src/cli.ts` to support parsing `--remote <url>`. 33c0066
- [x] Task: Refactor `run` function in `src/cli.ts` to use `RemoteEvaluator` when appropriate. 33c0066
- [ ] Task: Conductor - User Manual Verification 'Phase 2: CLI CLI Integration' (Protocol in workflow.md)

## Phase 3: Documentation and E2E
Finalize documentation and verify end-to-end functionality.

- [ ] Task: Create a comprehensive E2E test scenario with a mock server.
- [ ] Task: Update `docs/getting-started.md` and `README.md` with usage examples for `--remote`.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Documentation and E2E' (Protocol in workflow.md)
