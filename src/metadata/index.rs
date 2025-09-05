use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexToml {
    #[serde(rename = "hash-format")]
    pub hash_format: String,
    pub files: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub file: String,
    #[serde(rename = "hash-format")]
    pub hash_format: Option<String>,
    pub hash: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub metafile: bool,
    #[serde(default)]
    pub preserve: bool,
}
