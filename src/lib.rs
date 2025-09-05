pub mod cli;
pub mod metadata;
pub mod request;
pub mod task;
pub mod target;
pub mod hash;

use anyhow::{Context, Result};
use bytes::Bytes;
use reqwest::{Client, Url};
use serde::Deserialize;
use sha1::{Digest as Sha1DigestTrait, Sha1};
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;
use tracing::info;
use tokio::time::{sleep, Duration};

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn sha1_hex(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub(crate) async fn fetch_bytes(uri: &str) -> Result<Bytes> {
    // Support http(s) and file/local paths
    if let Ok(url) = uri.parse::<Url>() {
        match url.scheme() {
            "http" | "https" => {
                let client = request::client::build_http_client()?;
                let res = client.get(url).send().await?.error_for_status()?;
                let bytes = res.bytes().await?;
                Ok(bytes)
            }
            "file" => {
                let path = url
                    .to_file_path()
                    .map_err(|_| anyhow::anyhow!("invalid file:// path"))?;
                let mut f = tokio::fs::File::open(&path).await?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).await?;
                Ok(Bytes::from(buf))
            }
            _ => anyhow::bail!("unsupported scheme: {}", url.scheme()),
        }
    } else {
        // Treat as local path
        let p = Path::new(uri);
        let data = tokio::fs::read(p).await?;
        Ok(Bytes::from(data))
    }
}

pub(crate) async fn fetch_bytes_retry(uri: &str, attempts: usize) -> Result<Bytes> {
    let mut last_err: Option<anyhow::Error> = None;
    let mut delay = Duration::from_millis(500);
    for _ in 0..attempts {
        match fetch_bytes(uri).await {
            Ok(b) => return Ok(b),
            Err(e) => {
                last_err = Some(e);
                sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(8));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("download failed")))
}

pub(crate) fn join_uri(base: &str, rel: &str) -> Result<String> {
    if rel.starts_with("http://") || rel.starts_with("https://") || rel.starts_with("file:") {
        return Ok(rel.to_string());
    }
    if let Ok(url) = base.parse::<Url>() {
        match url.scheme() {
            "http" | "https" => {
                let joined = url
                    .join(rel)
                    .with_context(|| format!("failed to join url {} + {}", url, rel))?;
                return Ok(joined.to_string());
            }
            "file" => {
                let basep = url
                    .to_file_path()
                    .map_err(|_| anyhow::anyhow!("invalid base file url"))?;
                let joined = basep.parent().unwrap_or_else(|| Path::new(".")).join(rel);
                return Ok(format!("file://{}", joined.display()));
            }
            _ => {}
        }
    }
    // local path base
    let p = Path::new(base);
    let dir = if p.is_file() { p.parent().unwrap_or_else(|| Path::new(".")) } else { p };
    Ok(dir.join(rel).to_string_lossy().to_string())
}

#[allow(dead_code)]
fn hex_lower(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>() }

pub(crate) fn hash_hex(format: &str, data: &[u8]) -> Result<String> {
    match format.to_ascii_lowercase().as_str() {
        "sha1" => Ok(sha1_hex(data)),
        "sha256" => Ok(sha256_hex(data)),
        "sha512" => {
            use sha2::Sha512;
            let mut hasher = Sha512::new();
            use sha2::Digest;
            hasher.update(data);
            Ok(format!("{:x}", hasher.finalize()))
        }
        "md5" => {
            let digest = md5::compute(data);
            Ok(format!("{:x}", digest))
        }
        "murmur2" => {
            let h = crate::hash::murmur2::murmur2_hash(data);
            Ok(h.to_string())
        }
        other => anyhow::bail!("unsupported hash format: {}", other),
    }
}

pub(crate) fn hash_file_hex(format: &str, path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    hash_hex(format, &data)
}

// -------- CurseForge resolution (module scope) --------
#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct CfFile { id: i64, #[serde(rename = "modId")] mod_id: i64, #[serde(default)] downloadUrl: Option<String> }

#[derive(Debug, Clone, Deserialize)]
struct CfFilesResp { data: Vec<CfFile> }

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct CfModLinks { #[serde(default)] websiteUrl: String }

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct CfMod { id: i64, #[serde(default)] name: String, #[serde(default)] links: Option<CfModLinks> }

#[derive(Debug, Clone, Deserialize)]
struct CfModsResp { data: Vec<CfMod> }

pub(crate) fn cf_api_key() -> String {
    if let Ok(k) = std::env::var("CF_API_KEY") { return k; }
    let b64 = "JDJhJDEwJHNBWVhqblU1N0EzSmpzcmJYM3JVdk92UWk2NHBLS3BnQ2VpbGc1TUM1UGNKL0RYTmlGWWxh";
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

pub(crate) async fn cf_get_download_url(client: &Client, project_id: i64, file_id: i64) -> Result<std::result::Result<String, String>> {
    let key = cf_api_key();
    let files_req = serde_json::json!({ "fileIds": [file_id] });
    let resp = client
        .post("https://api.curseforge.com/v1/mods/files")
        .header("Accept", "application/json")
        .header("User-Agent", "packwiz-installer-rust")
        .header("X-API-Key", key.clone())
        .json(&files_req)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() { anyhow::bail!("curseforge files api error {}: {}", status, body); }
    let data: CfFilesResp = serde_json::from_str(&body)?;
    if let Some(cf_file) = data.data.first() {
        if let Some(url) = cf_file.downloadUrl.clone() { return Ok(Ok(url)); }
    }
    // fallback: manual link via /v1/mods
    let mods_req = serde_json::json!({ "modIds": [project_id] });
    let resp2 = client
        .post("https://api.curseforge.com/v1/mods")
        .header("Accept", "application/json")
        .header("User-Agent", "packwiz-installer-rust")
        .header("X-API-Key", key)
        .json(&mods_req)
        .send()
        .await?;
    let status2 = resp2.status();
    let body2 = resp2.text().await?;
    if !status2.is_success() { anyhow::bail!("curseforge mods api error {}: {}", status2, body2); }
    let mods: CfModsResp = serde_json::from_str(&body2)?;
    if let Some(m) = mods.data.first() {
        let base = m.links.as_ref().map(|l| l.websiteUrl.clone()).unwrap_or_default();
        let url = if base.is_empty() {
            format!("https://www.curseforge.com/projects/{}/files/{}", project_id, file_id)
        } else {
            format!("{}/files/{}", base.trim_end_matches('/'), file_id)
        };
        return Ok(Err(url));
    }
    Ok(Err(format!("https://www.curseforge.com/projects/{}/files/{}", project_id, file_id)))
}

// (types are used within task modules; not needed directly here)

pub async fn run(cfg: crate::cli::Cli) -> Result<()> {
    info!(?cfg, "starting packwiz-installer-rust");
    // Delegate to task::update to avoid duplication
    let pack_folder_for_update: PathBuf = cfg
        .pack_folder
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let opts_for_update = crate::task::update::Options {
        pack_uri: cfg.pack_uri.clone(),
        side: cfg.side,
        optional_mode: cfg.optional_mode,
        pack_folder: pack_folder_for_update,
        meta_file: cfg.meta_file.clone(),
    };
    return crate::task::update::run_update(opts_for_update).await;
}
