# Progress Log
Started: Sun Mar 29 00:45:12 EDT 2026

## Codebase Patterns
- (add reusable patterns here)

---
## [2026-03-29 00:49:01 EDT] - US-001: Scaffold rust catalog crate and workspace wiring
Thread: 
Run: 20260329-004512-10664 (iteration 1)
Run log: /Users/lars/Dev/qtip/.ralph/runs/run-20260329-004512-10664-iter-1.log
Run summary: /Users/lars/Dev/qtip/.ralph/runs/run-20260329-004512-10664-iter-1.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 56d0580 chore(rust-catalog): scaffold cargo workspace
- Post-commit status: `.todos/command_usage.jsonl, .todos/issues.db, conductor/tracks/qtip-guard-discovery_20260327/*, .agents/, prd.json`
- Verification:
  - Command: cargo fmt --all -- --check -> PASS
  - Command: cargo clippy --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --all --all-features -> PASS
  - Command: npm run rust:check -> PASS
  - Command: npm run build -> PASS
- Files changed:
  - .cargo/config.toml
  - .gitignore
  - AGENTS.md
  - CONTRIBUTING.md
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/qtip-catalog/Cargo.toml
  - crates/qtip-catalog/src/lib.rs
  - package.json
  - scripts/rust-quality-gates.sh
- What was implemented
  - Scaffolded `crates/qtip-catalog` as a Rust library crate and wired it into a root Cargo workspace.
  - Installed required crate dependencies with the exact `cargo add serde serde_yaml thiserror rayon tokio ignore globset tracing` command.
  - Added baseline Cargo aliases and repo-level scripts for lint/test/fmt quality gates.
  - Added setup and troubleshooting guidance for missing Rust toolchain and dependency resolution failures.
- **Learnings for future iterations:**
  - Patterns discovered
    - Repo-level npm scripts are a practical integration point for Rust quality gates in this mixed TS/Rust codebase.
  - Gotchas encountered
    - `cargo add` run before workspace wiring creates a member `Cargo.lock`; workspace should rely on root `Cargo.lock`.
  - Useful context
    - Existing repo had unrelated pre-run changes; story commit was kept scoped to US-001 files.
---
## [2026-03-29 00:53:15 EDT] - US-002: Define core domain types and trait ports
Thread: codex_19864
Run: 20260329-004512-10664 (iteration 2)
Run log: /Users/lars/Dev/qtip/.ralph/runs/run-20260329-004512-10664-iter-2.log
Run summary: /Users/lars/Dev/qtip/.ralph/runs/run-20260329-004512-10664-iter-2.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 9c7e64f feat(catalog): define core domain types and ports
- Post-commit status: `clean`
- Verification:
  - Command: cargo fmt --all -- --check -> PASS
  - Command: cargo clippy --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --all --all-features -> PASS
  - Command: npm run rust:check -> PASS
  - Command: npm run build -> PASS
- Files changed:
  - crates/qtip-catalog/src/lib.rs
  - .ralph/activity.log
  - .ralph/progress.md
- What was implemented
  - Added core catalog domain types: `ScenarioRef`, `ScenarioDocument`, `Scenario`, `CatalogError`, plus envelope aliases for refs/documents/scenarios.
  - Added trait ports `ScenarioSource` and `ScenarioCatalog` with explicit sync discovery (`discover_refs`/`discover`) and async loading (`load_document`/`load_documents`) boundaries.
  - Enforced source contract validation in default trait behavior to reject invalid records with structured `CatalogError` entries.
  - Added mock-source unit tests that load two in-memory files and verify negative contract handling for empty path and missing source id.
- **Learnings for future iterations:**
  - Patterns discovered
    - Default trait methods are a clean place to centralize adapter contract validation while keeping adapter implementations small.
  - Gotchas encountered
    - `thiserror` derive treats a field named `source` specially; manual `Display`/`Error` impl avoids accidental source-error wiring.
  - Useful context
    - A simple safe `Wake`-based helper can drive async trait futures in unit tests without adding an async runtime dependency to tests.
---
## [2026-03-29 00:58:20 EDT] - US-003: Implement filesystem deep discovery adapter
Thread: ses_653f6c
Run: 20260329-004512-10664 (iteration 3)
Run log: /Users/lars/Dev/qtip/.ralph/runs/run-20260329-004512-10664-iter-3.log
Run summary: /Users/lars/Dev/qtip/.ralph/runs/run-20260329-004512-10664-iter-3.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 78885a9 feat(catalog): implement filesystem deep discovery
- Post-commit status: `clean`
- Verification:
  - Command: npm run rust:check -> PASS
  - Command: cargo fmt --all -- --check -> PASS
  - Command: cargo clippy --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --all --all-features -> PASS
  - Command: npm run build -> PASS
- Files changed:
  - .agents/tasks/prd-rust-catalog.json
  - .ralph/activity.log
  - .todos/command_usage.jsonl
  - .todos/issues.db
  - crates/qtip-catalog/src/lib.rs
- What was implemented
  - Added `FileSystemSourceConfig` and `FileSystemSource` adapter with recursive root walking via `ignore::WalkBuilder`.
  - Added include/exclude/ignore glob compilation and matching, with `.gitignore` support and traversal-level filtering for explicit ignores.
  - Implemented non-fatal discovery error collection (including unreadable directories) while continuing traversal across other roots.
  - Added discovery acceptance tests for recursive nested YAML matches, explicit ignores, `.gitignore` handling, and unreadable-root continuation.
  - Hardened `load_document` against absolute paths and `..` traversal segments.
- **Learnings for future iterations:**
  - Patterns discovered
  - `ignore::WalkBuilder::filter_entry` is the cleanest way to prune ignored directories early while preserving non-fatal walk errors.
  - Gotchas encountered
  - `ignore::Error` in v0.4.25 has no `path()` helper; extracting paths requires matching enum variants (`WithPath`, `Loop`, nested wrappers).
  - Useful context
  - Running `npm run rust:check` early surfaces formatting drift quickly before clippy/test cycles.
---
