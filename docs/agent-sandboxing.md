# Agent Sandboxing & Evaluation Loops

The primary mission of **qtip** is to provide an immutable "Oracle" for autonomous AI agents. 

In a traditional CI/CD pipeline, the tests live in the same repository as the code. If an autonomous agent is given write access to the repository, it has the ability to:

1.  Modify the application code.
2.  Modify the test code to "fix" a failing test.

This creates a conflict of interest that compromises the integrity of the evaluation.

## The qtip Oracle Pattern

qtip solves this by decoupling the **Subject** (the agent's workspace) from the **Scenarios** (the source of truth).

### 1. The "Two-Repo" Strategy

**For maximum security, you should maintain two separate repositories:**

-   **Application Repository (e.g., `org/smith`)**: Where the AI agent has write permissions to develop features.
-   **Scenario Repository (e.g., `org/smith-scenarios`)**: Where the scenarios and acceptance criteria are stored. **The AI agent must have NO write permissions to this repository.**

### 2. One-Way Validation Moat
By storing scenarios in a **restricted remote repository** or at a **secure URL**, you create a one-way moat. When the agent triggers an evaluation, it is submitting its work to an external "Oracle." It can modify the subject, but it cannot touch the scenarios to "cheat" the test.

### 3. Traceability to Acceptance Criteria
Instead of returning cryptic stack traces, qtip returns failures mapped to **Acceptance Criteria (AC)**. This allows the agent to understand *why* its code is failing in business terms, guiding it toward a correct implementation without giving it the power to redefine success.

## Recommended Sandbox Architecture

```text
+-----------------------+           +--------------------------+
|  Application Repo     |           |  Scenario Repo (Locked)  |
|  (e.g., org/smith)    |           |  (e.g., org/smith-scenarios)
|                       |           |                          |
|  - App Code           |           |  - login.yaml            |
|  - GitHub Actions     |           |  - build.yaml            |
|                       |           |                          |
|  [ Agent Has Write ]  |           |  [ Agent READ-ONLY ]     |
+-----------+-----------+           +------------+-------------+
            |                                    |
            |           +----------------+       |
            +---------->|  qtip Runner   |<------+
                        +----------------+
                                |
                                v
                        Evaluation Result
                        (Pass/Fail vs. AC)
```

To implement a secure sandbox:

1.  **Restrict Scenario Access**: Host scenarios in a repository where the agent has no write permissions.
2.  **Locked Manifest**: Define the Subject Manifest in a protected GitHub Action workflow file (e.g., using `CODEOWNERS`).
3.  **Oracle Loop**: Configure the agent to use qtip-cli as its primary validation tool. 

By following this pattern, you ensure that the agent remains in a **Test-Driven Development (TDD) sandbox**, where the tests are the "Laws of the System" that cannot be broken or bypassed.
