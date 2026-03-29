# Contributing

## Rust Catalog Setup

The catalog crate lives at `crates/qtip-catalog` and is part of the root Cargo workspace.

Run Rust checks from repository root:

```bash
npm run rust:check
```

Or run quality gates directly:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
```

## Troubleshooting

- `cargo: command not found`:
  - Install Rust using `rustup`: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Reload shell env: `source "$HOME/.cargo/env"`
- Crates fail to download or resolve:
  - Check internet/proxy access to `https://crates.io` and `https://index.crates.io`
  - Retry with `cargo fetch` and `cargo update`
