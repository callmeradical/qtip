# Implementation Plan: Multi-Service Evaluation Support

## Phase 1: Update Schema and Models
- [x] Task: Update `SubjectManifestSchema` to include an optional `name` for interface identification. 19a9a06
    - [x] Add `name` field to `SubjectInterfaceSchema` in `src/models/subject-manifest.ts`. 19a9a06
- [x] Task: Update `ScenarioSchema` to include a `service` field in `ApiInteractionSchema`. 19a9a06
    - [x] Add `service` field to `ApiInteractionSchema` in `src/models/scenario.ts`. 19a9a06
- [x] Task: Conductor - User Manual Verification 'Phase 1: Update Schema and Models' (Protocol in workflow.md) 19a9a06

## Phase 2: Update ApiAdapter for Service Resolution
- [x] Task: Write Tests for Service Resolution in `ApiAdapter`. a080acf
    - [x] Create `src/tests/api-adapter.test.ts` to test targeting named services. a080acf
- [x] Task: Update `ApiAdapter` to resolve services by name. a080acf
    - [x] Modify `src/adapters/api-adapter.ts` to use the `service` name from the interaction to find the correct interface. a080acf
    - [x] Implement fallback to the first API interface if `service` is not specified. a080acf
- [x] Task: Conductor - User Manual Verification 'Phase 2: Update ApiAdapter for Service Resolution' (Protocol in workflow.md) a080acf

## Phase 3: Update CLI for Multi-Manifest Support
- [x] Task: Write Tests for CLI Multi-Manifest Loading. b5a7b90
    - [x] Refactor `src/cli.ts` to export a `run` function for testability. b5a7b90
    - [x] Create `src/tests/cli.test.ts` to test multi-manifest loading. b5a7b90
- [x] Task: Update `qtip-cli` to accept multiple manifest paths. b5a7b90
    - [x] Modify `src/cli.ts` to process multiple manifest files. b5a7b90
    - [x] Merge interfaces and capabilities from all loaded manifests for scenario resolution. b5a7b90
- [x] Task: Conductor - User Manual Verification 'Phase 3: Update CLI for Multi-Manifest Support' (Protocol in workflow.md) b5a7b90

## Phase 4: Full Multi-Service Scenario Validation
- [x] Task: Write Tests for Cross-Service Scenarios. 4b7cd32
    - [x] Create a complex scenario that hits two different services. 4b7cd32
    - [x] Verify that the `qtip` runner correctly switches between services. 4b7cd32
- [x] Task: Conductor - User Manual Verification 'Phase 4: Full Multi-Service Scenario Validation' (Protocol in workflow.md) 4b7cd32
