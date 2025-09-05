use anyhow::Result;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub fn build_http_client() -> Result<Client> {
    let client = ClientBuilder::new()
        .user_agent("packwiz-installer-rust/0.1")
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .pool_max_idle_per_host(8)
        .timeout(Duration::from_secs(30))
        .build()?;
    Ok(client)
}

