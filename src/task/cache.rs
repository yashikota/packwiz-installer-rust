use std::collections::BTreeSet;
use std::path::Path;
use serde_json::Value;

#[derive(Debug, Default, Clone)]
pub struct PreviousManifest {
    pub cached_paths: BTreeSet<String>,
    pub side: Option<crate::target::side::Side>,
}

pub fn load_previous(manifest_path: &Path) -> PreviousManifest {
    let mut out = PreviousManifest::default();
    if manifest_path.exists() {
        if let Ok(text) = std::fs::read_to_string(manifest_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&text) {
                out.side = val.get("cachedSide").and_then(|v| serde_json::from_value(v.clone()).ok());
                if let Some(obj) = val.get("cachedFiles").and_then(|v| v.as_object()) {
                    for k in obj.keys() { out.cached_paths.insert(k.clone()); }
                }
            }
        }
    }
    out
}

pub fn remove_unreferenced(previous: &PreviousManifest, new_paths: &BTreeSet<String>, pack_folder: &Path) {
    for removed in previous.cached_paths.difference(new_paths) {
        let _ = std::fs::remove_file(pack_folder.join(removed));
    }
}
