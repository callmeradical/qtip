# Track Specification: Multi-Service Evaluation Support

## Overview
Enable `qtip` to evaluate complex, multi-service applications by allowing it to load and coordinate multiple `Subject Manifests` in a single evaluation run. This allows scenarios to interact with different services (e.g., calling an `auth` service and then a `user` service).

## Functional Requirements
1.  **Unique Service Identification**: Ensure each `Subject Manifest` can be uniquely identified within an evaluation run using its `projectId`.
2.  **Multi-Manifest Loading**: Update `qtip-cli` and `qtip-server` to support loading multiple manifests from a list of paths or a directory.
3.  **Cross-Service Scenarios**: Add a `service` field to the `ApiInteractionSchema` to allow scenarios to explicitly target a service by its `projectId`.
4.  **Service Resolution**: Update the `ApiAdapter` to resolve the `baseUrl` by matching the scenario's `service` field against the loaded manifests' `projectId`.
5.  **Fallback Logic**: If no `service` is specified in a scenario, default to the first available API interface from the primary manifest.

## Non-Functional Requirements
- **Backward Compatibility**: Single-manifest evaluations must remain unaffected.
- **Validation**: Enforce unique `projectId` values when multiple manifests are loaded.

## Acceptance Criteria
- [ ] `qtip-cli` supports multiple manifest file paths as arguments.
- [ ] `ApiInteractionSchema` includes an optional `service` string.
- [ ] `ApiAdapter` correctly routes requests based on the `service` field.
- [ ] Clear error message if a scenario targets a non-existent service.
