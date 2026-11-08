use std::net::SocketAddr;

use tempfile::tempdir;
use url::Url;

use super::{Config, ConfigError, OidcConfig};

fn secret() -> Vec<u8> {
    b"test-session-secret-is-32-bytes!".to_vec()
}

#[test]
fn rejects_a_short_session_secret() {
    let root = tempdir().unwrap();
    let error = Config::new(
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        Url::parse("http://127.0.0.1:8792").unwrap(),
        b"too-short".to_vec(),
        false,
        None,
        "warp".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ConfigError::Invalid {
            name: "FACTORY_SESSION_SECRET",
            ..
        }
    ));
}

#[test]
fn rejects_a_path_as_a_repository_id() {
    let root = tempdir().unwrap();
    let error = Config::new(
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        Url::parse("http://127.0.0.1:8792").unwrap(),
        secret(),
        false,
        None,
        "../other".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ConfigError::Invalid {
            name: "FACTORY_REPO_ID",
            ..
        }
    ));
}

#[test]
fn trims_the_public_url_used_for_the_callback() {
    let root = tempdir().unwrap();
    let config = Config::new(
        "127.0.0.1:8792".parse().unwrap(),
        Url::parse("http://127.0.0.1:8792/").unwrap(),
        secret(),
        false,
        Some(OidcConfig {
            issuer: "https://id.example".to_string(),
            client_id: "company-factory".to_string(),
            client_secret: None,
        }),
        "warp".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap();
    assert_eq!(config.redirect_url(), "http://127.0.0.1:8792/callback");
}
