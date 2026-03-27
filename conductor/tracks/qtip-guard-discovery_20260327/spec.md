# Track Specification: qtip Guard Discovery & Hook Integration

## Overview
This track enhances the `qtip-guard` service to support the discovery of new source files and the installation of Git hooks for automated evaluation.

## Functional Requirements
- **Automatic Scenario Discovery**:
  - Enhance the `FileWatcher` to detect new file additions (using Chokidar's `add` event).
  - Automatically generate a qtip scenario in the secondary repository for any new file detected.
  - Initial generation will use a default template since no mapping exists yet in the configuration.
- **Git Hook Installation**:
  - Implement a CLI command (`qtip:hook install`) to install Git hooks (e.g., `pre-commit`, `pre-push`).
  - The hooks will trigger a full project evaluation using the qtip evaluation platform.
  - Provide a custom script for users to manually install or customize these hooks.
- **Evaluation on Hook**:
  - When the hook is triggered, it will execute the equivalent of `npm run qtip:evaluate` for all scenarios in the project.

## Non-Functional Requirements
- **Simplicity**: The hook installation process should be a single-command operation.
- **Robustness**: Hooks should fail gracefully if the evaluation service is not available.
- **Maintainability**: Hooks should be easy to remove or update.

## Acceptance Criteria
- [ ] The `qtip-guard` service automatically generates a new scenario YAML in the secondary repository when a new file is added to the SUT.
- [ ] Running `npm run qtip:hook install pre-commit` successfully installs a `pre-commit` hook in the `.git/hooks` directory.
- [ ] Committing changes correctly triggers the installed `pre-commit` hook, which runs the qtip evaluation.
- [ ] The Git hook allows the commit/push to proceed only if the qtip evaluation passes.
- [ ] A `qtip:hook install` command supports multiple hook types (`pre-commit`, `pre-push`).

## Out of Scope
- Intelligent mapping generation (this track uses a default template for discovered files).
- Selective evaluation (only running affected scenarios) during hook execution (initial implementation will run all).
