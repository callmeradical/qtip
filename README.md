# qtip — Scenario Evaluation Platform

**qtip** is a standardized mechanism for validating software systems using reusable scenario definitions. It decouples test definitions from application code, allowing the system under test (SUT) to describe itself while the platform decides how to evaluate it.

## 🚀 Key Features

- **Decoupled Scenarios**: Store and version scenarios in a separate repository or directory.
- **Manifest-Driven**: The SUT submits a "Subject Manifest" describing its capabilities and interfaces.
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

## 🛠️ Getting Started

### Prerequisites
- Node.js (v18+)
- Python (for documentation site)

### Installation
```bash
npm install
```

### Running the Platform
```bash
# Start the runner platform
npm start

# Development mode with hot-reload
npm run dev
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

## 🧪 Testing

```bash
npm test
```

## 📜 License
ISC
