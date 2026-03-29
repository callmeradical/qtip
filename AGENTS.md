## MANDATORY: Use td for Task Management

You must run td usage --new-session at conversation start (or after /clear) to see current work.
Use td usage -q for subsequent reads.

## Build/Test Quick Commands

- Rust quality gates: `npm run rust:check`
- Individual Rust checks: `npm run rust:fmt`, `npm run rust:clippy`, `npm run rust:test`
- TypeScript build: `npm run build`
