use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::env;
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub allowlist: GlobSet,
    pub timeout: Duration,
    pub max_bytes: usize,
}

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub allowlist: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub max_bytes: Option<usize>,
}

impl Config {
    pub fn from_env() -> Self {
        let allowlist_env = env::var("FETCH_ALLOWLIST").unwrap_or_else(|_| "https://*".to_string());
        let patterns: Vec<String> = allowlist_env.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let mut builder = GlobSetBuilder::new();
        for p in &patterns {
            // Allow host/path style patterns; prepend scheme wildcards if missing
            let gp = if p.starts_with("http://") || p.starts_with("https://") { p.clone() } else { format!("https://{}", p) };
            let glob = Glob::new(&gp).unwrap_or_else(|_| Glob::new("https://*").unwrap());
            builder.add(glob);
        }
        let allowlist = builder.build().unwrap();
        let timeout = Duration::from_millis(env::var("FETCH_TIMEOUT_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(15000));
        let max_bytes = env::var("FETCH_MAX_BYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(5 * 1024 * 1024);
        Self { allowlist, timeout, max_bytes }
    }

    pub fn is_allowed(&self, url: &Url) -> bool {
        // Match against full URL string
        self.allowlist.is_match(url.as_str())
    }
}
