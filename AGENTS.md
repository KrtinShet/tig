# Agent Instructions: Tig

Single Rust crate (not a workspace). Standard Cargo project with both a binary (`src/main.rs`) and library (`src/lib.rs`) target.

## Build & test

- `cargo build` — debug binary at `./target/debug/tig`
- `cargo test` — runs unit tests (in `src/*` `#[cfg(test)]` modules) plus `tests/integration_test.rs`
- `cargo build --release` — release binary at `./target/release/tig`
- No CI, custom `rustfmt.toml`, or `clippy.toml` exists; rely on defaults

## E2E demo

- `bash tests/e2e_demo.sh` runs an end-to-end bash demo
- **Must build the debug binary first** (`cargo build`). The script defaults to `/Users/krtinshet/Development/tessera-vcs/target/debug/tig` via the `TIG` env var; override with `TIG=./target/debug/tig` if needed
- The demo also requires `git` and `node` on PATH

## Runtime requirements

- `git` binary must be installed and on PATH. The `git_bridge` export/import feature and the integration/e2e tests invoke `git` directly

## Project layout

- `src/lib.rs` — library root; exposes `api` (programmatic API) and `cli` (command-line interface)
- `src/main.rs` — thin binary wrapper calling `tig::cli::run()`
- `tests/integration_test.rs` — integration tests using `tempfile` tempdirs and the public `tig::api::Tig` API
- `tests/e2e_demo.sh` — standalone bash demo

## Conventions

- Error handling uses `anyhow`; CLI uses `clap` derive macros
- Content-addressed object store (SHA-256). Internal state lives in `.tig/` under the project root
- Do not commit `.omc/`, `.claude/`, `progress.txt`, or `prd.json` (all in `.gitignore`)
