# Scenario Evaluation Platform — Product Requirements Document (PRD)

## 1. Problem Statement

Engineering teams need a standardized mechanism to validate that software systems meet defined acceptance criteria using reusable scenario definitions.

Current approaches are fragmented:

- Tests live inside the application repository
- Acceptance criteria are poorly mapped to test outcomes
- Multiple interaction methods exist (API, CLI, UI, logs)
- CI pipelines lack a unified way to evaluate results

The goal is to build a **Scenario Evaluation Platform** that:

- Executes reusable scenarios stored in a separate repository
- Evaluates whether a system meets defined acceptance criteria
- Supports multiple interaction methods (API, CLI, UI)
- Supports log-based validation
- Returns structured evaluation results via API

The **system under test (SUT)** should **not know what tests to run**.

Instead, the SUT **describes itself**, and the platform determines which scenarios apply.

---

# 2. Goals

## Primary Goals

Provide a platform that:

- Runs scenario definitions stored outside the application repo
- Maps test results to acceptance criteria
- Supports multiple interaction types
- Allows CI/CD pipelines to gate deployment
- Returns deterministic machine-readable results

## Secondary Goals

- Enable cross-project scenario reuse
- Allow teams to author scenarios without modifying application code
- Provide extensible interaction adapters

---

# 3. Non-Goals

For the MVP:

- Full UI test authoring interface
- Rich dashboard analytics
- Performance/load testing
- Complex distributed orchestration

---

# 4. Key Concepts

## Subject

The **software system being evaluated**.

The subject provides metadata, not tests.

Examples:

- identity-service
- smith-cli
- billing-api

The subject exposes a **Subject Manifest**.

---

## Scenario

A reusable test definition stored in the **Scenario Repository**.

A scenario defines:

- stimulus
- interaction method
- validation checks
- acceptance criteria

---

## Acceptance Criteria

Business requirements that must be met.

Example:

- AC-1: User receives access token
- AC-2: Login completes under 1s

Assertions map to criteria.

---

## Adapter

A runtime component that interacts with the subject.

Examples:

| Adapter | Purpose |
|--------|--------|
| API | HTTP / gRPC calls |
| CLI | Command execution |
| UI | Browser automation |
| Logs | Validate events |

---

# 5. High-Level Architecture

```
                +-----------------------+
                |   Scenario Repository |
                |-----------------------|
                | YAML Scenario Files   |
                +-----------+-----------+
                            |
                            v
+------------------------------------------------+
|            Scenario Runner Platform             |
|------------------------------------------------|
|                                                |
|  Scenario Resolver                             |
|  Scenario Executor                             |
|  Acceptance Evaluator                          |
|  Evidence Collector                            |
|                                                |
|  Adapters                                      |
|    - API                                       |
|    - CLI                                       |
|    - UI                                        |
|    - Logs                                      |
|                                                |
+------------+----------------+------------------+
             |                |
             v                v
      Subject System      Results Storage
      (System under test)
```

---

# 6. Core Repositories

## 1. Subject Repository

Contains application code.

Examples:

- github.com/org/identity-service
- github.com/org/smith

The application exposes a **subject manifest**.

---

## 2. Scenario Repository

Contains reusable scenarios.

Example structure:

```
/scenarios
  /auth
    login-success.yaml
    login-failure.yaml
  /cli
    build-command.yaml
  /ui
    register-user.yaml
```

---

## 3. Runner Platform

Responsible for:

- scenario resolution
- execution
- evaluation
- reporting

---

# 7. Real-World Workflow

## Step 1 — System Deploys

CI pipeline deploys service to a testing environment (e.g., QA or staging).

---

## Step 2 — System Submits Subject Manifest

Pipeline calls evaluation API.

```
POST /evaluate
```

Example payload:

```json
{
  "projectId": "identity-service",
  "repoUrl": "https://github.com/org/identity-service",
  "commit": "abc123",
  "environment": "qa",
  "interfaces": [
    {
      "type": "api",
      "baseUrl": "https://qa.identity.example.com"
    }
  ],
  "observability": {
    "logs": {
      "type": "file",
      "path": "/var/log/app.log"
    }
  },
  "capabilities": ["auth"]
}
```

The subject simply describes itself.

---

## Step 3 — Platform Resolves Scenarios

The platform loads scenario metadata and selects applicable ones.

Example resolution:

- AUTH-LOGIN-001
- AUTH-LOGIN-002
- AUTH-TOKEN-003

Selection is based on:

- capabilities
- interface type
- environment
- project ID
- version

---

## Step 4 — Scenario Execution

Each scenario is executed.

Example scenario: Valid user login.

Interaction example:

```
POST /login
```

Evidence collected includes:

- HTTP response
- response time
- logs

---

## Step 5 — Evidence Collection

Evidence may include:

- HTTP response
- response JSON
- response time
- stdout/stderr
- logs
- screenshots

---

## Step 6 — Acceptance Criteria Evaluation

Assertions are evaluated.

Example checks:

- status_code == 200
- access_token exists
- response_time < 1000ms

---

## Step 7 — Results Produced

The runner compiles the results.

Example:

- Scenario AUTH-LOGIN-001 PASSED
- Scenario AUTH-TOKEN-003 FAILED

---

## Step 8 — Results Returned

Example response:

```json
{
  "status": "failed",
  "summary": {
    "total": 3,
    "passed": 2,
    "failed": 1
  },
  "failures": [
    {
      "scenarioId": "AUTH-TOKEN-003",
      "acceptanceCriteria": [
        {
          "id": "AC-2",
          "description": "Token must expire in 1 hour"
        }
      ]
    }
  ]
}
```

---

## Step 9 — CI/CD Decision

Pipeline evaluates result:

```
if status == failed
  stop deployment
else
  promote build
```

---

# 8. Execution Lifecycle

```
Subject manifest submitted
        |
        v
Scenario resolution
        |
        v
Adapter preparation
        |
        v
Scenario interaction
        |
        v
Evidence collection
        |
        v
Check evaluation
        |
        v
Acceptance mapping
        |
        v
Result generation
```

---

# 9. API Contract

## Evaluate Subject

```
POST /evaluate
```

Example request:

```json
{
  "projectId": "identity-service",
  "environment": "qa",
  "interfaces": [
    {
      "type": "api",
      "baseUrl": "https://qa.identity.example.com"
    }
  ]
}
```

---

## Success Response

HTTP `200`

```json
{
  "status": "passed",
  "summary": {
    "total": 5,
    "passed": 5,
    "failed": 0
  }
}
```

---

## Failure Response

```json
{
  "status": "failed",
  "failures": [
    {
      "scenarioId": "AUTH-LOGIN-001",
      "acceptanceCriteria": ["AC-2"]
    }
  ]
}
```

---

# 10. Scenario Definition Format

Example scenario file:

```yaml
id: AUTH-LOGIN-001
name: Valid login returns token

applies_to:
  capabilities: [auth]
  interfaces: [api]

acceptance_criteria:
  - id: AC-1
    description: Login succeeds
  - id: AC-2
    description: Token returned

interaction:
  type: api
  request:
    method: POST
    path: /login
    body:
      username: test@example.com
      password: secret

checks:
  - type: status_code
    expected: 200
    acceptance_criteria: AC-1

  - type: json_path
    path: $.token
    exists: true
    acceptance_criteria: AC-2
```

---

# 11. Adapter Model

Adapters translate scenario steps into real interactions.

---

## API Adapter

Supports:

- REST
- GraphQL
- gRPC

Validations:

- status codes
- response body
- latency
- headers

---

## CLI Adapter

Runs commands.

Example:

```
smithctl build
```

Checks:

- exit code
- stdout
- stderr

---

## UI Adapter

Uses browser automation.

Suggested tooling:

- Playwright

Checks:

- DOM content
- navigation
- form submission
- screenshots

---

## Log Adapter

Validates system behavior indirectly.

Example checks:

- log contains event
- log does not contain error
- event emitted within time window

---

# 12. User Stories

## Developer

As a developer

I want my system evaluated automatically

So that I know it meets acceptance criteria.

---

## QA Engineer

As a QA engineer

I want reusable scenarios stored separately from application code

So that I can validate multiple projects consistently.

---

## Platform Engineer

As a platform engineer

I want scenario evaluation to return machine-readable results

So that CI/CD pipelines can gate deployments.

---

## Product Owner

As a product owner

I want failures mapped to acceptance criteria

So that I know which requirements were not met.

---

# 13. Acceptance Criteria

## Scenario Resolution

The system must:

- select scenarios based on capability
- filter scenarios by interface type
- respect environment constraints

---

## Execution

The platform must:

- execute scenarios deterministically
- collect execution evidence
- support multiple adapters

---

## Evaluation

The platform must:

- evaluate assertions
- map assertions to acceptance criteria
- return structured failure details

---

## API

The platform must:

- expose evaluation API
- return structured JSON results
- support CI integration

---

# 14. MVP Scope

## Included

- Subject manifest submission
- Scenario repository
- Scenario resolution engine
- API adapter
- CLI adapter
- Log validation
- Acceptance criteria mapping
- Results API

---

## Deferred

- UI testing
- Web UI for results
- historical analytics
- distributed runners
- advanced artifact storage

---

# Design Principle

**The system under test declares what it is.**  
**The platform decides how to test it.**

