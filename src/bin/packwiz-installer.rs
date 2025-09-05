use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging with RUST_LOG override, default to info
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env_filter).init();

    let cli = packwiz_installer::cli::Cli::parse();
    packwiz_installer::run(cli).await
}
