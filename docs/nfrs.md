# Non-Functional Requirements (NFRs)

The following non-functional requirements apply to the **qtip** Scenario Evaluation Platform.

## 1. Performance
- **Latency**: Scenario resolution must complete under 100ms.
- **Overhead**: Adapter execution should add minimal overhead to the actual interaction (excluding network/command execution time).

## 2. Scalability
- **Concurrency**: The platform must support concurrent execution of multiple scenarios for a single subject.
- **Project Isolation**: Results from different project evaluations must be isolated.

## 3. Reliability
- **Deterministic Execution**: Given the same environment and inputs, scenario execution must be deterministic.
- **Fault Tolerance**: Adapter failures (e.g., connection timeout) must be caught and reported as failures, not platform crashes.

## 4. Security
- **Data Protection**: Sensitive data (secrets, tokens) must not be logged or stored in evidence without masking.
- **Execution Sandboxing**: (Future) CLI adapter commands should run in a restricted environment.

## 5. Maintainability
- **Extensible Adapters**: Adding a new adapter type requires minimal code changes to the core runner.
- **Scenario Versioning**: The platform must eventually support loading different versions of scenarios based on the subject's version.
