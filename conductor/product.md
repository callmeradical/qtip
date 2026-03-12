# qtip - Scenario Evaluation Platform

## Initial Concept
qtip is a standardized mechanism for validating software systems using reusable scenario definitions. It decouples test definitions from application code, allowing the system under test (SUT) while the platform decides how to evaluate it.

## Product Vision
To provide a secure, "oracle-like" feedback loop for autonomous software engineering agents, ensuring high-integrity validation of codebase changes without allowing agents to modify the criteria for success.

## Target Users
- AI Software Engineering Agents (e.g., Gemini CLI) requiring objective validation.
- DevOps Engineers building high-integrity CI/CD pipelines.
- Security Researchers testing agent sandboxing and integrity.

## Key Features
- **One-Way Validation Moat:** Immutability of scenarios ensured by remote or restricted storage.
- **Subject Manifests:** SUTs describe their capabilities and interfaces via JSON.
- **Multi-Adapter Support:** Built-in adapters for API, CLI, and Log-based interactions.
- **Acceptance Criteria Mapping:** Links low-level checks to high-level business requirements.
- **CI/CD Integration:** Native GitHub Action support for deployment gating.

## Strategic Goals
1. Establish an industry-standard protocol for agent-independent software validation.
2. Minimize the cost of manual Acceptance Criteria verification.
3. Provide real-time, high-signal feedback to autonomous agents during development loops.
