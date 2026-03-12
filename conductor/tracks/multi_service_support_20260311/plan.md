# Implementation Plan: Multi-Service Evaluation Support

## Phase 1: Update Schema and Models
- [ ] Task: Update `SubjectManifestSchema` to include an optional `name` for interface identification.
    - [ ] Add `name` field to `SubjectInterfaceSchema` in `src/models/subject-manifest.ts`.
- [ ] Task: Update `ScenarioSchema` to include a `service` field in `ApiInteractionSchema`.
    - [ ] Add `service` field to `ApiInteractionSchema` in `src/models/scenario.ts`.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Update Schema and Models' (Protocol in workflow.md)

## Phase 2: Update ApiAdapter for Service Resolution
- [ ] Task: Write Tests for Service Resolution in `ApiAdapter`.
    - [ ] Create `src/tests/api-adapter.test.ts` to test targeting named services.
- [ ] Task: Update `ApiAdapter` to resolve services by name.
    - [ ] Modify `src/adapters/api-adapter.ts` to use the `service` name from the interaction to find the correct interface.
    - [ ] Implement fallback to the first API interface if `service` is not specified.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Update ApiAdapter for Service Resolution' (Protocol in workflow.md)

## Phase 3: Update CLI for Multi-Manifest Support
- [ ] Task: Write Tests for CLI Multi-Manifest Loading.
    - [ ] Create `src/tests/cli-multi-manifest.test.ts`.
- [ ] Task: Update `qtip-cli` to accept multiple manifest paths.
    - [ ] Modify `src/cli.ts` to process multiple manifest files.
    - [ ] Merge interfaces and capabilities from all loaded manifests for scenario resolution.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Update CLI for Multi-Manifest Support' (Protocol in workflow.md)

## Phase 4: Full Multi-Service Scenario Validation
- [ ] Task: Write Tests for Cross-Service Scenarios.
    - [ ] Create a complex scenario that hits two different services.
    - [ ] Verify that the `qtip` runner correctly switches between services.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Full Multi-Service Scenario Validation' (Protocol in workflow.md)
