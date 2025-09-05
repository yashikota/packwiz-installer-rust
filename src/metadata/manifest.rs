use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ManifestFile {
    pub packFileHash: Option<String>,
    pub indexFileHash: Option<String>,
    pub cachedFiles: serde_json::Map<String, serde_json::Value>,
    pub cachedSide: crate::target::side::Side,
}
