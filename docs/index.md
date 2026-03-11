# Scenario Evaluation Platform (qtip)

Welcome to the documentation for **qtip**, a standardized mechanism to validate software systems using reusable scenario definitions.

## Overview

Engineering teams need a standardized way to validate that software systems meet defined acceptance criteria. qtip provides a platform that:

- Executes reusable scenarios stored outside the application repository.
- Evaluates whether a system meets defined acceptance criteria.
- Supports multiple interaction methods (API, CLI, Logs).
- Returns structured evaluation results via API.

## Core Design Principle

**The system under test (SUT) declares what it is. The platform decides how to test it.**

## Key Components

- **Subject**: The software system being evaluated (provides a Manifest).
- **Scenario**: A reusable test definition.
- **Adapter**: A runtime component that interacts with the subject (API, CLI, Logs).
- **Runner**: The platform that orchestrates resolution, execution, and evaluation.
