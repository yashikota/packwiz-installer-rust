# packwiz-installer-rust

Experimental Rust CLI port of packwiz-installer. It fetches `pack.toml`, verifies and fetches `index.toml`, downloads files, validates hashes, cleans up removed files, and writes a deterministic `packwiz.json` compatible with the original installer.

## Build
- Debug: `cargo build`
- Release: `cargo build --release`

## Usage
- `target/release/packwiz-installer [FLAGS] <pack.toml URI|path>`

Flags
- `--side <client|server|both>`: Install side (default: `client`).
- `--pack-folder <path>`: Target folder for downloaded files and manifest (default: current directory).
- `--multimc-folder <path>`: Accepted for compatibility; currently unused by the Rust CLI.
- `--meta-file <file>`: Manifest file name relative to the pack folder (default: `packwiz.json`).
- `--optional-mode <default|all|none>`: Optional mods handling (default: `default`).
- `--timeout <secs>`: Seconds to wait for optional prompts (accepted; no interactive UI in Rust version).
- `--title <string>`: Accepted for compatibility; ignored by the Rust CLI.

Examples
- `cargo run --release -- --side client --pack-folder ./pack --meta-file packwiz.json https://example.com/pack.toml`
- Local file: `cargo run -- --side server ./path/to/pack.toml`
