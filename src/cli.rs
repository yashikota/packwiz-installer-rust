use crate::destination::side::Side;
use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OptionalMode {
    Default,
    All,
    None,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "packwiz-installer",
    version,
    about = "Rust port of packwiz-installer (CLI only)"
)]
pub struct Cli {
    /// Side to install mods from (client/server/both)
    #[arg(short = 's', long = "side", value_enum, default_value_t = Side::Client)]
    pub side: Side,

    /// Folder to install the pack to (defaults to the JAR directory in Java impl; here default is current dir)
    #[arg(long = "pack-folder")]
    pub pack_folder: Option<PathBuf>,

    /// The MultiMC pack folder (defaults to the parent of the pack directory in Java impl)
    #[arg(long = "multimc-folder")]
    pub multimc_folder: Option<PathBuf>,

    /// JSON file to store pack metadata, relative to the pack folder (defaults to packwiz.json)
    #[arg(long = "meta-file", default_value = "packwiz.json")]
    pub meta_file: String,

    /// Seconds to wait before automatically launching when asking about optional mods (defaults to 10)
    #[arg(short = 't', long = "timeout", default_value_t = 10u64)]
    pub timeout_secs: u64,

    /// How to treat optional mods: default (use pack defaults), all (enable all), none (disable all)
    #[arg(long = "optional-mode", value_enum, default_value_t = OptionalMode::Default)]
    pub optional_mode: OptionalMode,

    /// Title of the installer window (ignored in Rust CLI, accepted for compatibility)
    #[arg(long = "title", action = ArgAction::Set)]
    pub title: Option<String>,

    /// pack.toml URI/path to install from
    pub pack_uri: String,
}
