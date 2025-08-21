use crate::config::Config;
use anyhow::{anyhow, bail, Context, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use bytes::Bytes;
use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use url::Url;

#[derive(Clone)]
pub struct FetchTool {
    client: Client,
    cfg: Config,
}

impl FetchTool {
    pub fn new(cfg: Config) -> Self {
        let client = Client::builder()
            .user_agent("fetch-mcp-rust/0.1")
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .tcp_nodelay(true)
            .pool_max_idle_per_host(2)
            .build()
            .expect("build reqwest client");
        Self { client, cfg }
    }

    pub async fn call(&self, args: Value) -> Result<Value> {
        let input: FetchInput = serde_json::from_value(args).context("Invalid fetch args")?;
        let url = Url::parse(&input.url).context("Invalid URL")?;
        if !(url.scheme() == "http" || url.scheme() == "https") {
            bail!("Only http and https are allowed");
        }
        if !self.cfg.is_allowed(&url) {
            bail!("URL not allowed by allowlist");
        }

        let method = input.method.unwrap_or_else(|| "GET".to_string());
        let method = Method::from_bytes(method.as_bytes()).unwrap_or(Method::GET);
        let mut req = self.client.request(method, url);

        if let Some(hmap) = input.headers {
            for (k, v) in hmap.into_iter() {
                req = req.header(k, v);
            }
        }

        if let Some(body) = input.body {
            match body {
                Body::Text(s) => { req = req.body(s); }
                Body::Json(v) => { req = req.json(&v); }
            }
        }

        let timeout = input.timeout_ms.map(Duration::from_millis).unwrap_or(self.cfg.timeout);
        let max_bytes = input.max_bytes.unwrap_or(self.cfg.max_bytes);

        let resp = tokio::time::timeout(timeout, req.send()).await.map_err(|_| anyhow!("request timed out"))??;
        let status = resp.status().as_u16();
        let headers = collect_headers(&resp);
        let (body_bytes, truncated) = read_limited(resp, max_bytes).await?;

        let body_base64 = BASE64.encode(&body_bytes);
        let result = json!({
            "status": status,
            "headers": headers,
            "body": {
                "type": "base64",
                "data": body_base64,
                "truncated": truncated,
            }
        });
        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Body {
    Text(String),
    Json(serde_json::Value),
}

#[derive(Debug, Deserialize)]
pub struct FetchInput {
    pub url: String,
    pub method: Option<String>,
    pub headers: Option<std::collections::BTreeMap<String, String>>,
    pub body: Option<Body>,
    pub timeout_ms: Option<u64>,
    pub max_bytes: Option<usize>,
}

fn collect_headers(resp: &Response) -> Vec<(String, String)> {
    resp.headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect()
}

async fn read_limited(mut resp: Response, max: usize) -> Result<(Bytes, bool)> {
    use tokio::io::AsyncReadExt;
    let mut stream = resp.bytes_stream();
    let mut collected = bytes::BytesMut::with_capacity(std::cmp::min(max, 64 * 1024));
    let mut total = 0usize;
    use futures::StreamExt;
    let mut truncated = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if total + chunk.len() > max {
            let remain = max.saturating_sub(total);
            collected.extend_from_slice(&chunk[..remain]);
            truncated = true;
            break;
        } else {
            collected.extend_from_slice(&chunk);
            total += chunk.len();
        }
    }
    Ok((collected.freeze(), truncated))
}
