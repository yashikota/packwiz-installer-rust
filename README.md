# packwiz-installer-rust (CLI only)

Rust port skeleton of packwiz-installer. Fetches `pack.toml`, verifies and fetches `index.toml`, downloads files (URL mode), validates hashes, and writes `packwiz.json`.

Usage
- Build: `cargo build` (requires network to fetch crates)
- Run: `cargo run -- --side client path/to/pack.toml`
- Options: `--pack-folder`, `--multimc-folder`, `--meta-file`, `--timeout`, `--title` (accepted, ignored)

Notes
- Supported hash formats: `sha1`, `sha256`, `sha512`, `md5`, `murmur2`.
- Supported download modes: `url`, `curseforge` (uses CurseForge API; manual fallback when required).
- Output manifest: `packwiz.json` in `--pack-folder`.

Next
- Implement CurseForge mode and caching.
- Expand manifest fields for full compatibility.
