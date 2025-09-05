use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::PathBuf;
use futures::{stream, StreamExt};

use crate::metadata::pack::PackFile as PackFileToml;
use crate::metadata::index::IndexToml;
use crate::task::cache::{load_previous, remove_unreferenced};
use crate::task::download::{process_entry, EntryContext};

#[derive(Debug, Clone)]
pub struct Options {
    pub pack_uri: String,
    pub side: crate::target::side::Side,
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
    if !opts.pack_folder.exists() { std::fs::create_dir_all(&opts.pack_folder)?; }
    let manifest_path = opts.pack_folder.join(&opts.meta_file);

    // Load previous manifest for cleanup
    let prev = load_previous(&manifest_path);

    // Load index
    let (index_uri, index_hash_format, index_hash_expected) = if let Some(idx) = pack_toml.index.clone() {
        let file_uri = crate::join_uri(&opts.pack_uri, &idx.file)?;
        let fmt = idx.hash_format.unwrap_or_else(|| "sha256".into());
        let h = idx.hash;
        (file_uri, fmt, h)
    } else {
        anyhow::bail!("pack.toml is missing [index]")
    };
    let index_bytes = crate::fetch_bytes(&index_uri).await.with_context(|| "failed to fetch index file")?;
    if let Some(exp) = index_hash_expected.as_ref() {
        let got = crate::hash_hex(&index_hash_format, &index_bytes)?;
        if &got != exp { anyhow::bail!("index hash mismatch: got {}, expected {} (format {})", got, exp, index_hash_format); }
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
    let futs = index_toml.files.clone().into_iter().map(|e| process_entry(e, &ctx));
    let results: Vec<_> = stream::iter(futs).buffer_unordered(8).collect().await;
    let mut cached_files = serde_json::Map::new();
    let mut new_paths: BTreeSet<String> = BTreeSet::new();
    for r in results {
        if let Some(er) = r? { new_paths.insert(er.path.clone()); cached_files.insert(er.path, er.value); }
    }

    // Cleanup unreferenced
    remove_unreferenced(&prev, &new_paths, &opts.pack_folder);

    // Write manifest
    let manifest = crate::metadata::manifest::ManifestFile {
        packFileHash: Some(pack_hash_sha256),
        indexFileHash: index_hash_expected,
        cachedFiles: cached_files,
        cachedSide: opts.side,
    };
    let json = serde_json::to_vec_pretty(&manifest)?;
    std::fs::write(&manifest_path, json)?;

    Ok(())
}

fn super_hash_sha256(data: &[u8]) -> String { crate::sha256_hex(data) }
