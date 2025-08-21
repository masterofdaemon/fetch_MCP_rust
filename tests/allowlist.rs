use fetch_mcp_rust::config::Config;
use url::Url;

#[test]
fn allow_https_wildcard() {
    std::env::set_var("FETCH_ALLOWLIST", "https://example.com/*");
    let cfg = Config::from_env();
    assert!(cfg.is_allowed(&Url::parse("https://example.com/path").unwrap()));
    assert!(!cfg.is_allowed(&Url::parse("https://evil.com/").unwrap()));
}
