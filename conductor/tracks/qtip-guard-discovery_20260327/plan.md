# Implementation Plan: qtip Guard Discovery & Hook Integration

## Phase 1: Discovery Implementation
- [x] Task: Update `FileWatcher` to detect new file additions (Chokidar `add` event) [825b4bc]
- [x] Task: Integrate `add` event in the main `Guard` class to trigger scenario generation [b82c142]
- [~] Task: Implement a default scenario template for discovered files
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Discovery' (Protocol in workflow.md)

## Phase 2: CLI & Hook Installation
- [ ] Task: Implement a CLI command `qtip:hook install [hook-name]` (e.g., pre-commit, pre-push)
- [ ] Task: Implement the logic to write a script to the `.git/hooks/` directory
- [ ] Task: Ensure the installed hook script is executable (permissions)
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Hook Installation' (Protocol in workflow.md)

## Phase 3: Hook Execution & Logic
- [ ] Task: Implement the logic within the Git hook to trigger `npm run qtip:evaluate`
- [ ] Task: Ensure the Git hook correctly handles exit codes to block or allow the Git action
- [ ] Task: Conduct a full end-to-end test of the pre-commit hook with an evaluation run
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Hook Execution' (Protocol in workflow.md)

## Phase 4: Final Integration & Documentation
- [ ] Task: Finalize documentation for the new `qtip:hook` command and discovery features
- [ ] Task: Finalize integration tests for the complete discovery and hook workflow
- [ ] Task: Conductor - User Manual Verification 'Phase 4: CLI & Finalization' (Protocol in workflow.md)
