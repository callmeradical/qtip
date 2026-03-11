# Testability Matrix

This matrix describes the system's testability across different interaction adapters and validation mechanisms.

| Feature | API Adapter | CLI Adapter | Log Adapter |
| :--- | :---: | :---: | :---: |
| **Interaction** | HTTP (Axios) | Command Line (Exec) | File Reading |
| **Status Code Validation** | ✅ (HTTP) | ✅ (Exit Code) | ❌ |
| **Payload Validation** | ✅ (JSONPath) | ✅ (Regex match) | ✅ (Regex match) |
| **Response Time** | ✅ | ❌ | ❌ |
| **Headers/Metadata** | ✅ | ❌ | ❌ |
| **Artifact Collection** | ✅ (JSON) | ✅ (Stdout/Stderr) | ✅ (Matches) |
| **Acceptance Criteria Mapping** | ✅ | ✅ | ✅ |

## Extensibility

Additional adapters (e.g., UI with Playwright) can be integrated by:

1.  Implementing a new `Adapter` class in `src/adapters/`.
2.  Updating the `ScenarioExecutor` to handle the new `interaction` type.
3.  Adding relevant checks in `evaluateChecks`.
