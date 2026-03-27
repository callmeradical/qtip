# PRD: Local Agent Evaluation Experience

## 1. Problem Statement

Teams want to run qtip in local test environments and have coding agents invoke it as a reliable quality gate. Today, the core flow works, but the experience is fragmented across commands, docs, and setup steps. This causes confusion about the fastest path from "agent made a change" to "qtip pass/fail decision."

## 2. Objective

Deliver a clear, low-friction local workflow where:

- a developer can run qtip in less than 5 minutes from clone,
- an agent can invoke qtip with deterministic behavior,
- pass/fail output is easy for both humans and automation to consume.

## 3. Users

- Platform engineer: defines quality gates and wants predictable execution.
- Application developer: wants quick local validation before pushing.
- AI agent operator: wants an agent to run qtip and act on failures.

## 4. Goals

- Make local qtip invocation obvious and repeatable.
- Keep agent contract stable (inputs, outputs, exit codes).
- Support both standalone and delegated (remote server) local modes.
- Provide actionable failures tied to scenario/check context.

## 5. Non-Goals

- Building a full UI dashboard.
- Replacing CI workflows.
- Solving cloud-scale orchestration in this phase.

## 6. Experience Principles

- One obvious path: local setup and first run should be linear.
- Deterministic contract: same inputs produce same machine-readable output shape.
- Fast feedback: users can see what failed and why in one run.
- Agent-friendly: command is scriptable, non-interactive, and exits consistently.

## 7. User Stories and Acceptance Criteria

### US-001: Developer runs qtip locally in standalone mode

As a developer, I can run qtip locally against a manifest and scenarios directory and get a pass/fail decision.

Acceptance Criteria:

- AC-1.1 (positive): Given a valid manifest and matching scenarios, when I run `node dist/cli.js <manifest> --scenarios <dir>`, then qtip prints per-subject and global summaries and exits with code 0.
- AC-1.2 (negative): Given an invalid manifest path or malformed JSON, when I run the same command, then qtip prints a clear validation error and exits with code 1.

### US-002: Agent uses qtip as a deterministic quality gate

As an agent operator, I can have an agent execute qtip non-interactively and decide next actions from exit code and output.

Acceptance Criteria:

- AC-2.1 (positive): Given at least one failing scenario, when an agent runs qtip, then qtip exits with code 1 and includes scenario-level failure details.
- AC-2.2 (negative): Given a command with missing required args, when an agent runs qtip, then qtip prints usage guidance and exits with code 1.

### US-003: Platform engineer runs delegated local mode

As a platform engineer, I can run qtip in delegated mode against a local qtip server to mirror centralized gating behavior.

Acceptance Criteria:

- AC-3.1 (positive): Given a reachable local server, when I run `node dist/cli.js <manifest> --remote <url>`, then qtip POSTs to `<url>/evaluate`, prints the returned summary, and exits with the same pass/fail semantics as standalone mode.
- AC-3.2 (negative): Given an unreachable server or 4xx/5xx response, when I run delegated mode, then qtip prints a clear remote failure message and exits with code 1.

### US-004: Secrets stay safe in delegated mode

As a security-conscious operator, I can configure delegated auth without exposing tokens in output.

Acceptance Criteria:

- AC-4.1 (positive): Given `QTIP_REMOTE_TOKEN` is set, when delegated mode runs, then request headers include `Authorization: Bearer <token>`.
- AC-4.2 (negative): Given request failures and verbose logs, when delegated mode errors, then the raw token value is never printed to stdout/stderr.

## 8. End-to-End Flows

### Flow A: Local standalone evaluation

1. Install and build qtip.
2. Create or use a manifest file.
3. Run local evaluation command.
4. Receive scenario-level results and global summary.
5. Return exit code 0 (all pass) or 1 (any fail/error).

### Flow B: Local delegated evaluation

1. Start local qtip server.
2. Run CLI with `--remote <url>`.
3. CLI POSTs manifest to `/evaluate`.
4. Receive and print summary.
5. Return exit code 0 or 1 using same semantics as local mode.

## 9. Functional Requirements

### FR-1: Local command clarity

- Document canonical commands for:
  - build,
  - standalone local run,
  - delegated local run.

### FR-2: Agent invocation contract

- Command is non-interactive.
- Exit code behavior is explicit and stable.
- Output includes per-subject and global summary.

### FR-3: Input validation

- Invalid manifest JSON/path fails fast with clear error text.
- Missing scenarios directory returns clear guidance.

### FR-4: Delegated mode support

- `--remote` posts to `${baseUrl}/evaluate`.
- Handles network failures and non-2xx responses with clear messages.

### FR-5: Authentication for delegated mode (new)

- Support optional bearer token for remote requests.
- Token can be provided through env var for agent-safe automation.
- Token is never printed in logs.

## 10. Non-Functional Requirements

- Reliability: deterministic exit code contract.
- Security: sensitive token values are masked and never echoed.
- Performance: local startup + evaluation remains suitable for inner-loop use.
- Maintainability: no duplicated execution logic between local and remote paths.

## 11. Success Metrics

- Time-to-first-local-run under 5 minutes for new users.
- At least 95% of agent-triggered runs finish without manual command correction.
- Reduced support questions about "which qtip command should I run locally?"

## 12. Required Verification Commands

Implementation is considered complete only when each user story below is verified with the required commands and expected outcomes.

### Baseline quality checks (must pass)

```bash
npm run build
npm test -- --runInBand
npm run docs:build
```

### US-001 verification: standalone local run

```bash
# Positive path: expects exit code 0
node dist/cli.js '{"projectId":"local-smoke","environment":"local","interfaces":[{"type":"cli"}],"capabilities":["test"]}' --scenarios ./scenarios; echo "exit=$?"

# Negative path: expects exit code 1 with clear manifest/path error
node dist/cli.js ./does-not-exist.json --scenarios ./scenarios; echo "exit=$?"
```

### US-002 verification: agent deterministic gating

```bash
# Negative/failure path: expects exit code 1 with scenario failure details
node dist/cli.js '{"projectId":"agent-fail","environment":"qa","interfaces":[{"type":"api","baseUrl":"http://127.0.0.1:9"}],"capabilities":["auth"]}' --scenarios ./scenarios; echo "exit=$?"

# Missing-args path: expects usage output and exit code 1
node dist/cli.js; echo "exit=$?"
```

### US-003 verification: delegated local mode

Run in two terminals.

Terminal A:

```bash
npm start
```

Terminal B:

```bash
# Positive delegated path: expects same pass/fail semantics as standalone mode
node dist/cli.js '{"projectId":"remote-smoke","environment":"local","interfaces":[{"type":"cli"}],"capabilities":["test"]}' --remote http://localhost:3000/api/v1; echo "exit=$?"

# Negative delegated path: expects exit code 1 with remote error message
node dist/cli.js '{"projectId":"remote-smoke","environment":"local","interfaces":[{"type":"cli"}],"capabilities":["test"]}' --remote http://localhost:3999/api/v1; echo "exit=$?"
```

### US-004 verification: delegated auth and token safety (after FR-5 implementation)

```bash
QTIP_REMOTE_TOKEN="test-token" npm test -- --runInBand src/tests/remote-evaluator.test.ts src/tests/cli-remote.test.ts
```

Expected outcome:

- Auth header is included when `QTIP_REMOTE_TOKEN` is set.
- Raw token value does not appear in logs or error output.

## 13. MVP Scope

- Publish one canonical local workflow in docs.
- Preserve current pass/fail and summary behavior.
- Add delegated auth via optional bearer token env var.
- Add tests for auth header and token redaction behavior.

## 14. Acceptance Criteria

- AC-1: A user can run qtip locally using documented steps and get expected pass/fail behavior.
- AC-2: An agent can run qtip and reliably parse success/failure via exit code.
- AC-3: Delegated local mode works against a local server.
- AC-4: When auth token env var is set, delegated requests include bearer auth header.
- AC-5: Token value never appears in stdout/stderr logs.

## 15. Rollout Plan

- Phase 1: Documentation updates for local and delegated local flows.
- Phase 2: Delegated auth support + unit/integration tests.
- Phase 3: Agent-focused examples and prompt templates.

## 16. Risks and Mitigations

- Risk: command drift between docs and implementation.
  - Mitigation: validate all documented commands in CI.
- Risk: token leakage in logs.
  - Mitigation: centralized redaction and negative tests.
- Risk: confusion between local and delegated modes.
  - Mitigation: side-by-side examples and decision table.

## 17. Open Questions

- Should we add a `--format json` output mode for stricter agent parsing?
- Should auth use one env var for all remotes or support per-host config?
- Do we want a one-command bootstrap (`qtip doctor` or `qtip init`) in the next iteration?
