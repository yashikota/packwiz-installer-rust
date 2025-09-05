# Repository Guidelines

## Project Structure & Modules
- `src/bin/packwiz-installer.rs`: CLI entrypoint (Tokio async main).
- `src/lib.rs`: Library root wiring modules and `run`.
- `src/cli.rs`: Argument parsing with `clap`.
- `src/task/`: Update/download/cache pipeline.
- `src/metadata/`: TOML/JSON models (`pack.toml`, `index.toml`, `packwiz.json`).
- `src/request/`: HTTP client and errors.
- `src/hash/`: Hash algorithms and helpers.
- `target/`: Cargo build artifacts (ignored).

## Build, Test, Run
- Build (debug): `cargo build`
- Build (release): `cargo build --release`
- Run (release): `cargo run --release -- --side client --pack-folder ./pack <pack.toml URI|path>`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt --all`
- Tests: `cargo test` (no tests yet; see below to add)

## Coding Style & Conventions
- Rust 2024 edition; default rustfmt (4-space indent, max width per toolchain).
- Naming: `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- Errors: prefer `anyhow::Result` at boundaries and `thiserror` for domain errors.
- Tracing: use `tracing` with `RUST_LOG=info` (or `debug`) during dev.
- HTTP: `reqwest` with rustls; avoid blocking I/O in async paths.

## Testing Guidelines
- Unit tests: place in-module under `#[cfg(test)] mod tests { ... }`.
- Integration tests: `tests/` directory, files like `update_smoke.rs`.
- Focus areas: hashing (`hash::*`), URI join, index processing, optional-mod selection, and manifest writing.
- Run all: `cargo test` (ensure tests do not require network; mock or fixture files).

## Commit & Pull Requests
- Commits: imperative, concise subject; scope optional (e.g., `task: validate hashes before write`).
- PRs: include motivation, summary of changes, usage example (e.g., `cargo run ...`), and screenshots/log snippets when relevant.
- Requirements: builds on stable, `cargo fmt` and `cargo clippy` clean, added/updated tests for behavior changes.

## Security & Configuration
- Secrets: do not commit API keys. `CF_API_KEY` is read from env for CurseForge resolution.
- Logging: controlled via `RUST_LOG` (e.g., `RUST_LOG=packwiz_installer_rust=debug`).
- Network: keep downloads deterministic; verify hashes before writing; avoid widening timeouts without justification.

