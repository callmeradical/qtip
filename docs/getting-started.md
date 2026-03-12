# Getting Started with qtip

Welcome to **qtip**! The most efficient way to get started is by letting an AI agent handle the boilerplate. This guide covers both the agent-assisted and manual workflows, with a focus on the recommended **Two-Repo Strategy**.

## 1. Onboarding with AI (Recommended)

If you use **Gemini CLI**, you can leverage the built-in **qtip-manifest-helper** skill. This skill guides the agent in creating schema-valid project manifests and test scenarios.

### Install the Skill
```bash
gemini skills install qtip-manifest-helper.skill --scope user --consent
/skills reload
```

### Prompting the Agent
Once the skill is enabled, your agent becomes a **qtip** expert. You can use prompts like:

- *"Help me initialize a new scenario repository called 'smith-scenarios'."*
- *"Generate a qtip manifest for my 'smith' project that points to the 'smith-scenarios' repository."*
- *"Help me write a qtip scenario in my scenarios repo to validate that my login endpoint returns a 200 OK."*

## 2. Installation

If you're setting up the platform runner yourself, clone the repository and install dependencies:

```bash
git clone https://github.com/callmeradical/qtip.git
cd qtip
npm install
npm run build
```

## 3. Initialize your Scenarios (The Two-Repo Strategy)

For maximum security and to prevent AI agents from "cheating" by modifying tests, **qtip** recommends storing your scenarios in a separate, restricted repository.

1.  **Create a Scenario Repo**: e.g., `github.com/org/smith-scenarios`.
2.  **Add Scenarios**: Create YAML files in this repo.

`auth/login-success.yaml` (in your scenario repo)
```yaml
id: AUTH-01
name: "Successful Login"
applies_to:
  capabilities: ["auth"]
  interfaces: ["api"]
acceptance_criteria:
  - id: AC-1
    description: "User receives a 200 OK"
interaction:
  type: api
  request:
    method: POST
    path: /login
    body:
      username: "test@example.com"
      password: "password123"
checks:
  - type: status_code
    expected: 200
    acceptance_criteria: AC-1
```

## 4. Start the Runner Platform

Start the qtip server to listen for evaluation requests:

```bash
npm start
```

## 5. Submit a Subject Manifest

A **Subject Manifest** describes your project's identity and where its scenarios are located. Note that you can point `scenariosSource` to a local path, a git repo, or a remote URL.

```bash
curl -X POST http://localhost:3000/api/v1/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "projectId": "identity-service",
    "environment": "qa",
    "interfaces": [
      {
        "type": "api",
        "baseUrl": "https://qa-api.example.com"
      }
    ],
    "capabilities": ["auth"]
  }'
```

## 6. View the Results

When a manifest is submitted, qtip will:
1.  **Resolve**: Match the manifest's capabilities to existing scenarios.
2.  **Execute**: Use the appropriate adapter (API, CLI, or Logs) to interact with the subject.
3.  **Evaluate**: Compare the results against the defined checks.
4.  **Report**: Produce a structured JSON report mapping failures to Acceptance Criteria.

## 7. Modes of Operation

Depending on your workflow, you can use **qtip** in three primary ways:

### A. Local CLI (Standalone Runner)
Best for developers testing their services locally or simple CI scripts. In this mode, the `qtip` command is a standalone runner that processes manifests and scenarios directly on the machine.

*   **How it works:** You pass the manifest and scenarios directory as arguments.
*   **Example:**
    ```bash
    qtip ./manifest.json --scenarios ./my-scenarios
    ```

### B. Runner Platform (Centralized Service)
Best for large organizations with a central "Quality Gateway." The **Subject** (your application) is responsible for pushing its identity to the qtip server.

*   **How it works:** Your application's CI pipeline finishes a deployment and then `POST`s its manifest to the qtip server's endpoint.
*   **The Flow:**
    1.  Subject deploys to a test environment.
    2.  CI script calls: `curl -X POST http://qtip-server:3000/api/v1/evaluate -d @manifest.json`.
    3.  qtip server pulls the scenarios, executes the tests, and returns a JSON report.

### C. GitHub Action (CI/CD Integrated)
Best for teams using GitHub Actions who want "Quality Gating" built into their PRs.

*   **How it works:** The `callmeradical/qtip` action installs the runner inside the GitHub environment and executes the evaluation locally.
*   **Key Benefit:** Results are automatically posted to the **GitHub Step Summary** for easy visibility by reviewers.

## 8. Next Steps

- **Agent Sandboxing**: Learn why the [Two-Repo Strategy](./agent-sandboxing.md) is critical for AI-driven development.
- **GitHub Action**: Use qtip in CI/CD with our [GitHub Action](./README.md#github-action).
- **Testability Matrix**: Explore supported [adapters and checks](./testability.md).
