# High-Level Architecture

The platform follows a decoupled architecture where the system under test (SUT) is separated from the scenario definitions.

## Workflow

1.  **Subject Deploys**: CI pipeline deploys the service.
2.  **Subject Manifest Submitted**: The service calls the evaluation API (POST `/evaluate`) describing itself.
3.  **Scenario Resolution**: The platform selects applicable scenarios based on capabilities and interfaces.
4.  **Scenario Execution**: Each scenario is executed using the appropriate Adapter (API, CLI, Logs).
5.  **Evidence Collection**: Evidence (HTTP response, stdout, log matches) is collected.
6.  **Evaluation**: Assertions are checked against collected evidence.
7.  **Results**: Structured results mapping to Acceptance Criteria are returned.

## System Components

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
|    - Logs                                      |
|                                                |
+------------+----------------+------------------+
             |                |
             v                v
      Subject System      Results Storage
```

## Implementation Stack

- **Platform**: Node.js / TypeScript / Express
- **Schema Validation**: Zod
- **API Client**: Axios
- **JSON Validation**: JSONPath-Plus
- **Documentation**: Zensical
