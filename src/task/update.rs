use anyhow::{Context, Result};
use futures::{StreamExt, stream};
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::metadata::index::IndexToml;
use crate::metadata::pack::PackFile as PackFileToml;
use crate::task::cache::{load_previous, remove_unreferenced};
use crate::task::download::{EntryContext, process_entry};

#[derive(Debug, Clone)]
pub struct Options {
    pub pack_uri: String,
    pub side: crate::destination::side::Side,
    pub optional_mode: crate::cli::OptionalMode,
    pub pack_folder: PathBuf,
    pub meta_file: String,
}

pub async fn run_update(opts: Options) -> Result<()> {
    // Fetch pack.toml
    let pack_bytes = crate::fetch_bytes(&opts.pack_uri)
        .await
        .with_context(|| format!("failed to fetch pack file: {}", opts.pack_uri))?;
    let pack_hash_sha256 = super_hash_sha256(&pack_bytes);
    let pack_toml: PackFileToml = toml::from_str(std::str::from_utf8(&pack_bytes)?)
        .with_context(|| "failed to parse pack.toml")?;

    // Prepare paths
    if !opts.pack_folder.exists() {
        std::fs::create_dir_all(&opts.pack_folder)?;
    }
    let manifest_path = opts.pack_folder.join(&opts.meta_file);

    // Load previous manifest for cleanup
    let prev = load_previous(&manifest_path);

    // Load index
    let (index_uri, index_hash_format, index_hash_expected) =
        if let Some(idx) = pack_toml.index.clone() {
            let file_uri = crate::join_uri(&opts.pack_uri, &idx.file)?;
            let fmt = idx.hash_format.unwrap_or_else(|| "sha256".into());
            let h = idx.hash;
            (file_uri, fmt, h)
        } else {
            anyhow::bail!("pack.toml is missing [index]")
        };
    let index_bytes = crate::fetch_bytes(&index_uri)
        .await
        .with_context(|| "failed to fetch index file")?;
    if let Some(exp) = index_hash_expected.as_ref() {
        let got = crate::hash_hex(&index_hash_format, &index_bytes)?;
        if &got != exp {
            anyhow::bail!(
                "index hash mismatch: got {}, expected {} (format {})",
                got,
                exp,
                index_hash_format
            );
        }
    }
    let index_toml: IndexToml = toml::from_str(std::str::from_utf8(&index_bytes)?)
        .with_context(|| "failed to parse index.toml")?;

    // Process entries in parallel
    let http = crate::request::client::build_http_client()?;
    let ctx = EntryContext {
        pack_folder: opts.pack_folder.clone(),
        index_uri: index_uri.clone(),
        index_hash_format_default: index_toml.hash_format.clone(),
        side: opts.side,
        optional_mode: opts.optional_mode,
        http,
    };
    let futs = index_toml
        .files
        .clone()
        .into_iter()
        .map(|e| process_entry(e, &ctx));
    let results: Vec<_> = stream::iter(futs).buffer_unordered(8).collect().await;
    // Collect results into a lookup to allow insertion in index order
    let mut by_path: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    let mut new_paths: BTreeSet<String> = BTreeSet::new();
    for r in results {
        if let Some(er) = r? {
            new_paths.insert(er.path.clone());
            by_path.insert(er.path, er.value);
        }
    }
    // Build cached_files preserving existing order from previous manifest,
    // then append new files in index.toml order (mimics Kotlin's completion service behavior)
    let mut cached_files = serde_json::Map::new();

    // First, preserve order from existing manifest if it exists
    #[allow(clippy::collapsible_if)]
    if manifest_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&manifest_path) {
            if let Ok(existing_manifest) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(existing_files) = existing_manifest
                    .get("cachedFiles")
                    .and_then(|v| v.as_object())
                {
                    for existing_key in existing_files.keys() {
                        if let Some(v) = by_path.remove(existing_key) {
                            cached_files.insert(existing_key.clone(), v);
                        }
                    }
                }
            }
        }
    }

    // Then add any new files in index.toml order
    for index_entry in &index_toml.files {
        let entry_path = index_entry.file.clone();
        if let Some(v) = by_path.remove(&entry_path) {
            cached_files.insert(entry_path, v);
        }
    }

    // Cleanup unreferenced
    remove_unreferenced(&prev, &new_paths, &opts.pack_folder);

    // Write manifest
    let manifest = crate::metadata::manifest::ManifestFile {
        packFileHash: Some(crate::metadata::manifest::HashKV {
            type_: "sha256".into(),
            value: pack_hash_sha256,
        }),
        indexFileHash: index_hash_expected.map(|v| crate::metadata::manifest::HashKV {
            type_: index_hash_format.clone(),
            value: v,
        }),
        cachedFiles: cached_files,
        cachedSide: opts.side,
    };
    // Write compact JSON with a trailing newline
    let mut f = std::fs::File::create(&manifest_path)?;
    serde_json::to_writer(&mut f, &manifest)?;
    use std::io::Write as _;
    writeln!(&mut f)?;

    Ok(())
}

fn super_hash_sha256(data: &[u8]) -> String {
    crate::sha256_hex(data)
}
