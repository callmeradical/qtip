# Specification: CLI Client Mode

## Overview
This track implements a "Client Mode" for the qtip CLI, enabling users to delegate Subject Manifest evaluation to a remote qtip server. This allows for centralized quality gating and simplified integration for complex services like Smith.

## Functional Requirements
- **Remote Flag:** Add a `--remote <url>` flag to the CLI command-line arguments.
- **Delegated Evaluation:** If `--remote` is present, the CLI skips local scenario resolution and execution.
- **HTTP POST:** The CLI sends the manifest(s) via `POST` to `${url}/evaluate`.
- **Synchronous Feedback:** The CLI waits for the server response and displays the evaluation summary and failures.
- **Exit Codes:** The CLI exits with 0 on 'passed' and 1 on 'failed' or error, consistent with local behavior.

## Non-Functional Requirements
- **Consistency:** Remote evaluation results must be rendered using the same output format as local evaluations.
- **Robustness:** The CLI must provide clear error messages for network failures or server errors.
- **Timeout:** Initial implementation will use a default 60s timeout for the remote request.

## Acceptance Criteria
- [ ] `qtip <manifest> --remote <url>` sends valid JSON to the server.
- [ ] Server 'passed' response results in CLI exit code 0 and a "PASSED" summary.
- [ ] Server 'failed' response results in CLI exit code 1 and a "FAILED" summary with failure details.
- [ ] Network errors are caught and reported clearly to the user.

## Out of Scope
- Authentication/Authorization (API Keys, OAuth).
- Asynchronous evaluation (Fire and Forget).
- Support for non-HTTP transport.
