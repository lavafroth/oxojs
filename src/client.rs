use anyhow::Result;
use reqwest::Client;

use reqwest::header::HeaderMap;
use std::time::Duration;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn initialize(timeout: u64, user_agent: Option<String>) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        user_agent
            .unwrap_or(APP_USER_AGENT.to_string())
            .try_into()?,
    );
    Ok(Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(timeout))
        .build()?)
}
