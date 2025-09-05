use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashKV {
    #[serde(rename = "type")] pub type_: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ManifestFile {
    pub packFileHash: Option<HashKV>,
    pub indexFileHash: Option<HashKV>,
    pub cachedFiles: serde_json::Map<String, serde_json::Value>,
    pub cachedSide: crate::target::side::Side,
}
