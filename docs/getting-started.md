# Getting Started with qtip

Welcome to **qtip**! The most efficient way to get started is by letting an AI agent handle the boilerplate. This guide covers both the agent-assisted and manual workflows.

## 1. Onboarding with AI (Recommended)

If you use **Gemini CLI**, you can leverage the built-in **qtip-manifest-helper** skill. This skill guides the agent in creating schema-valid project manifests and test scenarios.

### Install the Skill
```bash
gemini skills install qtip-manifest-helper.skill --scope user --consent
/skills reload
```

### Prompting the Agent
Once the skill is enabled, your agent becomes a **qtip** expert. You can use prompts like:

- *"Generate a qtip manifest for a Node.js project called 'auth-service' that uses API validation on port 8080."*
- *"Help me write a qtip scenario to validate that my login endpoint returns a 200 OK."*

## 2. Installation

If you're setting up the platform runner yourself, clone the repository and install dependencies:

```bash
git clone https://github.com/callmeradical/qtip.git
cd qtip
npm install
npm run build
```

## 3. Initialize your Scenarios

The runner platform evaluates your system against YAML scenarios. These are usually stored in a `scenarios/` directory.

`scenarios/auth/login-success.yaml`
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

A **Subject Manifest** is a JSON object that describes your project's identity, interfaces, and capabilities. This is what triggers the evaluation.

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

## 7. Next Steps

- **GitHub Action**: Use qtip in CI/CD with our [GitHub Action](./README.md#github-action).
- **Agent Sandboxing**: Learn how qtip provides a [secure evaluation loop](./agent-sandboxing.md) for AI agents.
- **Testability Matrix**: Explore supported [adapters and checks](./testability.md).
