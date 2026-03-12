# Implementation Plan: Multi-Manifest Run Support (Revised)

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

## Phase 5: Update CLI Run Loop for Multi-Subject Evaluation
- [x] Task: Update `src/cli.ts` to iterate through manifests instead of merging them. 7213
- [x] Task: Ensure unique project execution context for each subject. 7213
- [x] Task: Write tests for sequential execution of multiple manifest files. 7213

## Phase 6: Aggregate Multi-Manifest Reporting
- [x] Task: Update `run` function to return an aggregated result object for all subjects. 7213
- [x] Task: Update CLI output to provide a distinct, clean summary per subject and a global total. 7213
- [x] Task: Conductor - User Manual Verification 'Phase 6: Aggregate Multi-Manifest Reporting' (Protocol in workflow.md) 8593

## Phase 7: Batch Run Validation
- [x] Task: Execute `qtip` with a directory of manifests. 10203
- [x] Task: Verify that failures in one manifest do not block the evaluation of others. 10473
- [x] Task: Conductor - User Manual Verification 'Phase 7: Batch Run Validation' (Protocol in workflow.md) 10473
