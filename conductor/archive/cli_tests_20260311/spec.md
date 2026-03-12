# Track Specification: Implement comprehensive unit tests for the CLI adapter

## Goal
To ensure the CLI adapter correctly handles command execution, exit code verification, and stdout/stderr processing, specifically for the agent-sandboxing use case.

## Objectives
- Achieve 100% test coverage for `src/adapters/cli-adapter.ts`.
- Verify behavior with both successful and failing commands.
- Test handling of timeouts and large outputs.

## Requirements
- Use `jest` and `ts-jest` for testing.
- Mock the child process or use a safe environment for executing real CLI commands.
- Verify `AcceptanceCriteria` mapping for CLI checks.
