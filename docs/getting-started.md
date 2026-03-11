# Getting Started with qtip

Welcome to **qtip**! This guide will help you set up the Scenario Evaluation Platform and run your first evaluation.

## 1. Installation

First, clone the repository and install the dependencies:

```bash
git clone https://github.com/callmeradical/qtip.git
cd qtip
npm install
```

## 2. Initialize your Scenarios

qtip looks for scenario definitions in a `scenarios/` directory (or a remote URL). Create your first scenario file:

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

## 3. Start the Runner Platform

Start the qtip server to listen for evaluation requests:

```bash
npm start
```

The server will be running at `http://localhost:3000`.

## 4. Submit a Subject Manifest

Now, you (the "Subject") need to describe yourself to qtip. You do this by submitting a **Subject Manifest**.

Open a new terminal and run:

```bash
curl -X POST http://localhost:3000/api/v1/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "projectId": "my-identity-service",
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

## 5. View the Results

qtip will:
1.  **Resolve**: Find the `AUTH-01` scenario because your manifest has the `auth` capability and an `api` interface.
2.  **Execute**: Call `https://qa-api.example.com/login`.
3.  **Evaluate**: Check if the response was `200`.
4.  **Report**: Return a JSON response showing if your project passed or failed.

## 6. Next Steps

- **GitHub Action**: Integrate qtip into your CI/CD pipeline using the [GitHub Action](./README.md#github-action).
- **Agent Sandboxing**: Learn how to use qtip as a [secure evaluation loop](./agent-sandboxing.md) for AI agents.
- **Testability Matrix**: See which [adapters and checks](./testability.md) are supported.
