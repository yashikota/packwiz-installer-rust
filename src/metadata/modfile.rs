use serde::de::{self, Deserializer, Unexpected, Visitor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModToml {
    pub name: String,
    pub filename: String,
    #[serde(default)]
    pub side: crate::destination::side::Side,
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

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub enum DownloadMode {
    #[default]
    Url,
    Curseforge,
}

impl<'de> Deserialize<'de> for DownloadMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ModeVisitor;
        impl<'de> Visitor<'de> for ModeVisitor {
            type Value = DownloadMode;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str("a download mode string: \"\", \"url\" or \"metadata:curseforge\"")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "" | "url" => Ok(DownloadMode::Url),
                    "metadata:curseforge" => Ok(DownloadMode::Curseforge),
                    other => Err(E::invalid_value(Unexpected::Str(other), &self)),
                }
            }
        }
        deserializer.deserialize_any(ModeVisitor)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModUpdate {
    #[serde(default)]
    pub curseforge: Option<CfUpdate>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CfUpdate {
    #[serde(rename = "project-id")]
    pub project_id: i64,
    #[serde(rename = "file-id")]
    pub file_id: i64,
}
