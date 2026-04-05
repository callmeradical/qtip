# qtip — Scenario Evaluation Platform

**qtip** is a standardized mechanism for validating software systems using reusable scenario definitions. It decouples test definitions from application code, allowing the system under test (SUT) to describe itself while the platform decides how to evaluate it.

## The "Oracle" Pattern: Agent Sandboxing

qtip was specifically designed to solve the "Agent Integrity" problem in autonomous software engineering. When an AI agent is tasked with modifying a codebase, there is a risk that it might "cheat" by altering the tests to pass its own broken code.

**qtip enforces a one-way validation moat:**

1.  **Immutability**: By loading scenarios from a **remote URL** or a **restricted repository**, you ensure the agent cannot redefine "success."
2.  **Decoupling**: The agent works in the application repository, but the "definition of truth" lives in qtip.
3.  **Oracle Feedback**: The agent submits its work and receives a deterministic, objective report on which Acceptance Criteria (AC) were met and which failed.

This makes qtip the ideal **Evaluation Loop** for AI-driven development.

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap callmeradical/tap
brew install qtip
```

### From source

```bash
cargo install --path crates/qtip-cli
```

### Node.js (legacy TypeScript CLI)

```bash
npm install && npm run build
node dist/cli.js <manifest.json> --scenarios ./scenarios
```

## Quick Start

**1. Create a manifest** describing your system (`manifest.json`):

```json
{
  "projectId": "my-service",
  "environment": "ci",
  "interfaces": [
    { "type": "api", "baseUrl": "http://localhost:8080" },
    { "type": "cli" }
  ],
  "capabilities": ["auth", "build"]
}
```

**2. Write scenarios** in YAML (`scenarios/auth/login.yaml`):

```yaml
id: AUTH-001
name: Valid Login
applies_to:
  capabilities: [auth]
  interfaces: [api]
acceptance_criteria:
  - id: AC-1
    description: Login returns 200
  - id: AC-2
    description: Token is present
interaction:
  type: api
  request:
    method: POST
    path: /login
    body: { "user": "test" }
checks:
  - type: status_code
    expected: 200
    acceptance_criteria: AC-1
  - type: json_path
    path: $.token
    exists: true
    acceptance_criteria: AC-2
```

**3. Run the evaluation:**

```bash
qtip manifest.json --scenarios ./scenarios
```

Output:
```
Evaluating Subject: my-service
my-service: Resolved 1 scenarios for project my-service

  - AUTH-001: AUTH-001 ... PASSED

Summary (my-service): 1 passed, 0 failed, 1 total.
```

Exit code 0 on success, 1 on any failure — ready for CI gating.

## Adapters

qtip supports three interaction types, selected per-scenario:

| Adapter | Interaction Type | Evidence Collected |
|---------|-----------------|-------------------|
| **API** | `api` | HTTP status, JSON body, headers |
| **CLI** | `cli` | Exit code, stdout, stderr |
| **Logs** | `logs` | Line matching against log files |

### Check Types

| Check | Works With | Description |
|-------|-----------|-------------|
| `status_code` | API, CLI | Compare HTTP status or exit code |
| `json_path` | API | Query JSON response with JSONPath, check existence or value |
| `stdout` | CLI | Assert stdout contains expected string |
| `stderr` | CLI | Assert stderr contains expected string |
| `log_contains` | Logs | Assert a log event was found |
| `log_not_contains` | Logs | Assert a log event was NOT found |
| `loop_state` | CLI/API JSON evidence | Assert workflow loop state at `$.state` or `$.record.state` |
| `github_pr_exists` | CLI JSON evidence | Assert at least one PR from `gh pr list --json` matches configured filters |
| `github_labels_match` | CLI JSON evidence | Assert issue labels contain (or exactly equal) expected labels |
| `github_comment_contains` | CLI JSON evidence | Assert at least one comment body matches a regex pattern |

## Architecture

```
manifest.json ──> qtip ──> scenarios/*.yaml
                    |
        ┌───────────┼───────────┐
        v           v           v
  qtip-catalog  qtip-resolver  qtip-executor
  (discover)    (filter)       (run + check)
                                    |
                             ┌──────┼──────┐
                             v      v      v
                           CLI    Log    API
                          adapter adapter adapter
```

The platform is built as a Rust workspace with four crates:

| Crate | Purpose |
|-------|---------|
| `qtip-catalog` | Scenario file discovery with glob patterns, `.gitignore` support, concurrent loading |
| `qtip-resolver` | Matches scenarios to subject capabilities, interfaces, and environments |
| `qtip-executor` | Check evaluation engine, adapter trait, and concrete CLI/Log/API adapters |
| `qtip-cli` | Binary that wires the pipeline together |

## Scenario Resolution

Scenarios are filtered based on three dimensions:

1. **Capabilities** — at least one of the scenario's capabilities must appear in the manifest
2. **Interfaces** — at least one of the scenario's interface types must appear in the manifest
3. **Environments** (optional) — if a scenario specifies environments, the manifest's environment must match

Scenarios with no `environments` field match any environment.

## CI/CD Integration

### GitHub Action

```yaml
steps:
  - uses: actions/checkout@v4
  - name: Evaluate Scenarios
    uses: callmeradical/qtip@main
    with:
      manifest: |
        {
          "projectId": "my-service",
          "environment": "ci",
          "interfaces": [{ "type": "cli" }],
          "capabilities": ["build"]
        }
      scenarios-directory: './scenarios'
```

### Direct CLI in CI

```yaml
steps:
  - uses: actions/checkout@v4
  - name: Install qtip
    run: cargo install --path crates/qtip-cli
  - name: Run evaluation
    run: qtip manifest.json --scenarios scenarios
```

## Development

### Prerequisites

- Rust toolchain (`rustup`)
- Node.js 18+ (for the legacy TypeScript components)

### Building and Testing

```bash
# Run all Rust tests (67 tests across 3 library crates)
cargo test --all

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Build the CLI binary
cargo build --release --package qtip-cli

# Run the legacy TypeScript tests
npm install && npm test
```

### Releasing

Releases are automated via GitHub Actions. To cut a release:

```bash
git tag v0.3.0
git push origin v0.3.0
```

This triggers cross-compilation for macOS (amd64/arm64) and Linux (amd64/arm64), creates a GitHub Release with tarballs and checksums, and updates the Homebrew formula in `callmeradical/homebrew-tap`.

## Documentation

Full documentation including the Architecture Deep-Dive and NFRs is built using Zensical:

```bash
npm run docs:build
npm run docs:serve
```

Workflow authoring reference:

- [Workflow Scenarios Guide](docs/workflow-scenarios.md)

## License

ISC
