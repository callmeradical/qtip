# Track Specification: Multi-Manifest Evaluation Support (Revised)

## Overview
Enable `qtip` to perform batch evaluations of multiple software subjects in a single execution. While the initial focus was on "multi-service" (merging interfaces), this revision prioritizes the ability to supply multiple distinct manifests and receive a combined report for all subjects.

## Functional Requirements
1.  **Multi-Manifest CLI Arguments**: Update `qtip-cli` to accept multiple manifest paths or a directory containing manifest files.
2.  **Independent Evaluation Loop**: Instead of merging manifests into one, iterate through each loaded manifest and execute its applicable scenarios independently.
3.  **Multi-Subject Report Summary**: Generate a high-level summary that tracks pass/fail status across all evaluated projects (e.g., "3 subjects evaluated, 2 passed, 1 failed").
4.  **Cross-Service Orchestration**: (Optional) Allow one "master" evaluation that can resolve scenarios across the set of loaded subjects, enabling true integration tests between independent manifests.

## Acceptance Criteria
- [ ] `qtip-cli` can take a list of manifests and run them sequentially.
- [ ] CLI output shows a clear section for each manifest's results.
- [ ] The final summary aggregate results across all manifests.
- [ ] The runner doesn't stop on the first subject's failure (unless configured otherwise).
