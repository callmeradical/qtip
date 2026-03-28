# Implementation Plan: qtip Guard Discovery & Hook Integration

## Phase 1: Discovery Implementation [checkpoint: 61b26b8]
- [x] Task: Update `FileWatcher` to detect new file additions (Chokidar `add` event) [825b4bc]
- [x] Task: Integrate `add` event in the main `Guard` class to trigger scenario generation [b82c142]
- [x] Task: Implement a default scenario template for discovered files [1421faf]
- [x] Task: Conductor - User Manual Verification 'Phase 1: Discovery' (Protocol in workflow.md) [61b26b8]

## Phase 2: CLI & Hook Installation [checkpoint: a31f29b]
- [x] Task: Implement a CLI command `qtip:hook install [hook-name]` (e.g., pre-commit, pre-push) [c63613c]
- [x] Task: Implement the logic to write a script to the `.git/hooks/` directory [1e852fb]
- [x] Task: Ensure the installed hook script is executable (permissions) [1e852fb]
- [x] Task: Conductor - User Manual Verification 'Phase 2: Hook Installation' (Protocol in workflow.md) [a31f29b]

## Phase 3: Hook Execution & Logic [checkpoint: 24f78ca]
- [x] Task: Implement the logic within the Git hook to trigger `npm run qtip:evaluate` [1e852fb]
- [x] Task: Ensure the Git hook correctly handles exit codes to block or allow the Git action [1e852fb]
- [x] Task: Conduct a full end-to-end test of the pre-commit hook with an evaluation run [f7718a8]
- [x] Task: Conductor - User Manual Verification 'Phase 3: Hook Execution' (Protocol in workflow.md) [24f78ca]

## Phase 4: Final Integration & Documentation [checkpoint: 76949c5]
- [x] Task: Finalize documentation for the new `qtip:hook` command and discovery features [d1e3d52]
- [x] Task: Finalize integration tests for the complete discovery and hook workflow [83a95a7]
- [x] Task: Conductor - User Manual Verification 'Phase 4: CLI & Finalization' (Protocol in workflow.md) [76949c5]
