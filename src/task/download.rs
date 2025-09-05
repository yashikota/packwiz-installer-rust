use anyhow::{Context, Result};
use reqwest::Client;
use std::path::PathBuf;

use crate::metadata::index::IndexEntry;
use crate::metadata::modfile::{DownloadMode, ModToml};

#[derive(Debug)]
pub struct EntryContext {
    pub pack_folder: PathBuf,
    pub index_uri: String,
    pub index_hash_format_default: String,
    pub side: crate::target::side::Side,
    pub optional_mode: crate::cli::OptionalMode,
    pub http: Client,
}

#[derive(Debug)]
pub struct EntryResult { pub path: String, pub value: serde_json::Value }

pub async fn process_entry(entry: IndexEntry, ctx: &EntryContext) -> Result<Option<EntryResult>> {
    let file_hash_fmt_owned = entry.hash_format.clone().unwrap_or(ctx.index_hash_format_default.clone());
    if entry.metafile {
        let mod_uri = crate::join_uri(&ctx.index_uri, &entry.file)?;
        let mod_bytes = crate::fetch_bytes_retry(&mod_uri, 3).await.with_context(|| format!("failed to fetch metafile: {}", entry.file))?;
        let mod_toml: ModToml = toml::from_str(std::str::from_utf8(&mod_bytes)?)
            .with_context(|| "failed to parse mod metadata")?;
        let include_side = match (ctx.side, mod_toml.side) {
            (crate::target::side::Side::Both, _) => true,
            (crate::target::side::Side::Client, crate::target::side::Side::Server) => false,
            (crate::target::side::Side::Server, crate::target::side::Side::Client) => false,
            _ => true,
        };
        let include_opt = match ctx.optional_mode {
            crate::cli::OptionalMode::Default => !mod_toml.option.optional || mod_toml.option.default_value,
            crate::cli::OptionalMode::All => true,
            crate::cli::OptionalMode::None => !mod_toml.option.optional,
        };
        if !(include_side && include_opt) {
            let temp = entry.alias.clone().unwrap_or_else(|| mod_toml.filename.clone());
            let _ = std::fs::remove_file(ctx.pack_folder.join(&temp));
            return Ok(None);
        }
        let dest_rel_val = entry.alias.clone().unwrap_or_else(|| mod_toml.filename.clone());
        let dest_abs = ctx.pack_folder.join(&dest_rel_val);
        match mod_toml.download.mode {
            DownloadMode::Url => {
                let mut got = None;
                if dest_abs.exists() {
                    if let Ok(h) = crate::hash_file_hex(&mod_toml.download.hash_format, &dest_abs) { if h == mod_toml.download.hash { got = Some(h); } }
                }
                if got.is_none() {
                    let url = mod_toml.download.url.ok_or_else(|| anyhow::anyhow!("download.url missing"))?;
                    let url_abs = crate::join_uri(&mod_uri, &url)?;
                    let bytes = crate::fetch_bytes_retry(&url_abs, 3).await.with_context(|| format!("failed to download {}", url_abs))?;
                    let h = crate::hash_hex(&mod_toml.download.hash_format, &bytes)?;
                    if h != mod_toml.download.hash { anyhow::bail!("mod hash mismatch for {}: got {}, expected {} ({})", mod_toml.name, h, mod_toml.download.hash, mod_toml.download.hash_format); }
                    if let Some(parent) = dest_abs.parent() { std::fs::create_dir_all(parent).ok(); }
                    std::fs::write(&dest_abs, &bytes)?;
                    got = Some(h);
                }
                let mut file_obj = serde_json::Map::new();
                let mut hash_obj = serde_json::Map::new();
                hash_obj.insert("type".into(), serde_json::Value::String(mod_toml.download.hash_format.clone()));
                hash_obj.insert("value".into(), serde_json::Value::String(got.unwrap()));
                file_obj.insert("hash".into(), serde_json::Value::Object(hash_obj));
                file_obj.insert("isOptional".into(), serde_json::Value::Bool(mod_toml.option.optional));
                file_obj.insert("optionValue".into(), serde_json::Value::Bool(mod_toml.option.default_value));
                let linked_hash_fmt = entry.hash_format.as_ref().unwrap_or(&file_hash_fmt_owned);
                let linked_val = entry.hash.clone();
                let mut linked_obj = serde_json::Map::new();
                linked_obj.insert("type".into(), serde_json::Value::String(linked_hash_fmt.clone()));
                linked_obj.insert("value".into(), serde_json::Value::String(linked_val));
                file_obj.insert("linkedFileHash".into(), serde_json::Value::Object(linked_obj));
                file_obj.insert("cachedLocation".into(), serde_json::Value::String(dest_rel_val.clone()));
                Ok(Some(EntryResult { path: dest_rel_val, value: serde_json::Value::Object(file_obj) }))
            }
            DownloadMode::Curseforge => {
                let cf = mod_toml.update.curseforge.ok_or_else(|| anyhow::anyhow!("curseforge update section missing"))?;
                match crate::cf_get_download_url(&ctx.http, cf.project_id, cf.file_id).await? {
                    Ok(url) => {
                        let mut got = None;
                        if dest_abs.exists() {
                            if let Ok(h) = crate::hash_file_hex(&mod_toml.download.hash_format, &dest_abs) { if h == mod_toml.download.hash { got = Some(h); } }
                        }
                        if got.is_none() {
                            let bytes = crate::fetch_bytes_retry(&url, 3).await.with_context(|| format!("failed to download {}", url))?;
                            let h = crate::hash_hex(&mod_toml.download.hash_format, &bytes)?;
                            if h != mod_toml.download.hash { anyhow::bail!("mod hash mismatch for {}: got {}, expected {} ({})", mod_toml.name, h, mod_toml.download.hash, mod_toml.download.hash_format); }
                            if let Some(parent) = dest_abs.parent() { std::fs::create_dir_all(parent).ok(); }
                            std::fs::write(&dest_abs, &bytes)?;
                            got = Some(h);
                        }
                        let mut file_obj = serde_json::Map::new();
                        let mut hash_obj = serde_json::Map::new();
                        hash_obj.insert("type".into(), serde_json::Value::String(mod_toml.download.hash_format.clone()));
                        hash_obj.insert("value".into(), serde_json::Value::String(got.unwrap()));
                        file_obj.insert("hash".into(), serde_json::Value::Object(hash_obj));
                        file_obj.insert("isOptional".into(), serde_json::Value::Bool(mod_toml.option.optional));
                        file_obj.insert("optionValue".into(), serde_json::Value::Bool(mod_toml.option.default_value));
                        let linked_hash_fmt = entry.hash_format.as_ref().unwrap_or(&file_hash_fmt_owned);
                        let linked_val = entry.hash.clone();
                        let mut linked_obj = serde_json::Map::new();
                        linked_obj.insert("type".into(), serde_json::Value::String(linked_hash_fmt.clone()));
                        linked_obj.insert("value".into(), serde_json::Value::String(linked_val));
                        file_obj.insert("linkedFileHash".into(), serde_json::Value::Object(linked_obj));
                        file_obj.insert("cachedLocation".into(), serde_json::Value::String(dest_rel_val.clone()));
                        Ok(Some(EntryResult { path: dest_rel_val, value: serde_json::Value::Object(file_obj) }))
                    }
                    Err(manual_url) => {
                        tracing::warn!("CurseForge API excluded file; manual download needed: {}", manual_url);
                        let mut file_obj = serde_json::Map::new();
                        let linked_hash_fmt = entry.hash_format.as_ref().unwrap_or(&file_hash_fmt_owned);
                        let linked_val = entry.hash.clone();
                        let mut linked_obj = serde_json::Map::new();
                        linked_obj.insert("type".into(), serde_json::Value::String(linked_hash_fmt.clone()));
                        linked_obj.insert("value".into(), serde_json::Value::String(linked_val));
                        file_obj.insert("linkedFileHash".into(), serde_json::Value::Object(linked_obj));
                        file_obj.insert("isOptional".into(), serde_json::Value::Bool(mod_toml.option.optional));
                        file_obj.insert("optionValue".into(), serde_json::Value::Bool(mod_toml.option.default_value));
                        file_obj.insert("cachedLocation".into(), serde_json::Value::String(dest_rel_val.clone()));
                        Ok(Some(EntryResult { path: dest_rel_val, value: serde_json::Value::Object(file_obj) }))
                    }
                }
            }
        }
    } else {
        let file_uri = crate::join_uri(&ctx.index_uri, &entry.file)?;
        let dest_rel_val = entry.alias.clone().unwrap_or_else(|| entry.file.clone());
        let dest_abs = ctx.pack_folder.join(&dest_rel_val);
        let mut got = None;
        if dest_abs.exists() {
            if let Ok(h) = crate::hash_file_hex(&file_hash_fmt_owned, &dest_abs) { if h == entry.hash { got = Some(h); } }
        }
        if entry.preserve && dest_abs.exists() {
            // keep
        } else if got.is_none() {
            let bytes = crate::fetch_bytes_retry(&file_uri, 3).await.with_context(|| format!("failed to download {}", entry.file))?;
            let h = crate::hash_hex(&file_hash_fmt_owned, &bytes)?;
            if h != entry.hash { anyhow::bail!("file hash mismatch for {}: got {}, expected {} ({})", entry.file, h, entry.hash, file_hash_fmt_owned); }
            if let Some(parent) = dest_abs.parent() { std::fs::create_dir_all(parent).ok(); }
            std::fs::write(&dest_abs, &bytes)?;
            got = Some(h);
        }
        let mut file_obj = serde_json::Map::new();
        let mut hash_obj = serde_json::Map::new();
        hash_obj.insert("type".into(), serde_json::Value::String(file_hash_fmt_owned.clone()));
        hash_obj.insert("value".into(), serde_json::Value::String(got.unwrap_or_else(|| entry.hash.clone())));
        file_obj.insert("hash".into(), serde_json::Value::Object(hash_obj));
        file_obj.insert("cachedLocation".into(), serde_json::Value::String(dest_rel_val.clone()));
        Ok(Some(EntryResult { path: dest_rel_val, value: serde_json::Value::Object(file_obj) }))
    }
}

