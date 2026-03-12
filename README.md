# qtip — Scenario Evaluation Platform

**qtip** is a standardized mechanism for validating software systems using reusable scenario definitions. It decouples test definitions from application code, allowing the system under test (SUT) to describe itself while the platform decides how to evaluate it.

## 🛡️ The "Oracle" Pattern: Agent Sandboxing

qtip was specifically designed to solve the "Agent Integrity" problem in autonomous software engineering. When an AI agent is tasked with modifying a codebase, there is a risk that it might "cheat" by altering the tests to pass its own broken code.

**qtip enforces a one-way validation moat:**

1.  **Immutability**: By loading scenarios from a **remote URL** or a **restricted repository**, you ensure the agent cannot redefine "success."
2.  **Decoupling**: The agent works in the application repository, but the "definition of truth" lives in qtip.
3.  **Oracle Feedback**: The agent submits its work and receives a deterministic, objective report on which Acceptance Criteria (AC) were met and which failed.

This makes qtip the ideal **Evaluation Loop** for AI-driven development.

## 🚀 Key Features
...

- **Decoupled Scenarios**: Store and version scenarios in a separate, restricted repository (e.g., `org/smith-scenarios`) rather than the application repo.
- **Manifest-Driven**: The SUT (e.g., `org/smith`) submits a "Subject Manifest" describing its capabilities and interfaces.
- **Multi-Adapter Support**:
  - **API**: Validate REST/JSON responses using JSONPath.
  - **CLI**: Validate exit codes and stdout/stderr.
  - **Logs**: Validate system behavior by scanning log files for events.
- **Acceptance Criteria Mapping**: Automatically map low-level assertions to high-level business requirements.
- **CI/CD Ready**: Provides machine-readable JSON results for deployment gating.

## 🏗️ Architecture

```
Subject manifest submitted
        |
        v
Scenario resolution (matches capabilities + interfaces)
        |
        v
Adapter execution (API, CLI, or Logs)
        |
        v
Evidence collection & Check evaluation
        |
        v
Acceptance criteria mapping
        |
        v
Structured Results (Passed/Failed)
```

## 🛠️ Getting Started (Agent-First)

The most effective way to use **qtip** is to let an AI agent (like Gemini CLI) handle the boilerplate for you. This repository includes a specialized skill to automate the creation of your project's identity and test scenarios.

### 1. Install the qtip Skill
If you are using Gemini CLI, install the helper skill to your user profile:

```bash
gemini skills install qtip-manifest-helper.skill --scope user --consent
/skills reload
```

### 2. Let the Agent Author your Manifest
Instead of writing JSON by hand, simply ask your agent:
> *"Help me create a qtip manifest for my project 'smith'. It's a CLI tool that builds microservices."*

The agent will use the `qtip-manifest-helper` skill to:
- Identify the correct **interfaces** (e.g., `cli`).
- Suggest relevant **capabilities** (e.g., `build`, `scaffold`).
- Generate a schema-valid JSON manifest.

### 3. Authoring Scenarios with AI
You can also ask the agent to author complex scenarios based on your requirements:
> *"Create a qtip scenario that validates smith's build command. It should check that the exit code is 0 and stdout contains 'Success'."*

---

## 🚀 Running the Platform

### Installation
```bash
npm install
npm run build
```

### CLI Usage
You can run qtip evaluations locally or delegate them to a remote server.

```bash
# Local Evaluation
node dist/cli.js <manifest.json> --scenarios ./scenarios

# Remote Evaluation (Delegated to a central server)
node dist/cli.js <manifest.json> --remote http://localhost:3000/api/v1
```

### Starting the Server
```bash
# Start the runner platform
npm start
```

### Evaluating a Subject
Submit a manifest to the `/evaluate` endpoint:

```bash
curl -X POST http://localhost:3000/api/v1/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "projectId": "identity-service",
    "environment": "qa",
    "interfaces": [{ "type": "api", "baseUrl": "https://api.example.com" }],
    "capabilities": ["auth"]
  }'
```

## 📄 Scenario Definition

Scenarios are defined in YAML. Example (`scenarios/auth/login.yaml`):

```yaml
id: AUTH-001
name: Valid Login
applies_to:
  capabilities: [auth]
  interfaces: [api]
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
```

## 📚 Documentation

The full documentation, including the Architecture Deep-Dive, Testability Matrix, and NFRs, is built using **Zensical**.

```bash
# Build documentation
npm run docs:build

# Preview documentation locally
npm run docs:serve
```

## 🐙 GitHub Action

You can use **qtip** directly in your CI pipelines with our GitHub Action.

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

The action will:
1. Resolve applicable scenarios.
2. Execute interactions (API, CLI, Logs).
3. Post a **Job Summary** table with the results.
4. Fail the build if any acceptance criteria are not met.

## 📜 License
ISC
