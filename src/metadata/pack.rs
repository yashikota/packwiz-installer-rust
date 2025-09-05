use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFile {
    pub name: Option<String>,
    #[serde(rename = "pack-format")]
    pub pack_format: Option<serde_json::Value>,
    pub index: Option<IndexFileLoc>,
    #[serde(default)]
    pub versions: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexFileLoc {
    pub file: String,
    #[serde(rename = "hash-format")]
    pub hash_format: Option<String>,
    pub hash: Option<String>,
}
