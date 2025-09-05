use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModToml {
    pub name: String,
    pub filename: String,
    #[serde(default)]
    pub side: crate::target::side::Side,
    pub download: ModDownload,
    #[serde(default)]
    pub option: ModOption,
    #[serde(default)]
    pub update: ModUpdate,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModOption {
    #[serde(default)]
    pub optional: bool,
    #[serde(default, rename = "default")]
    pub default_value: bool,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModDownload {
    pub url: Option<String>,
    #[serde(rename = "hash-format")]
    pub hash_format: String,
    pub hash: String,
    #[serde(default)]
    pub mode: DownloadMode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DownloadMode {
    Url,
    Curseforge,
}
impl Default for DownloadMode { fn default() -> Self { DownloadMode::Url } }

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModUpdate { #[serde(default)] pub curseforge: Option<CfUpdate> }

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CfUpdate { #[serde(rename = "project-id")] pub project_id: i64, #[serde(rename = "file-id")] pub file_id: i64 }
