use std::net::SocketAddr;

use tempfile::tempdir;
use url::Url;

use super::{CompanySignIn, Config, ConfigError, OidcConfig};

fn sign_in(domain: &str) -> CompanySignIn {
    CompanySignIn {
        oidc: OidcConfig {
            issuer: "https://accounts.google.com".to_string(),
            client_id: "company-factory".to_string(),
            client_secret: Some("dedicated-client-secret".to_string()),
        },
        allowed_domain: domain.to_string(),
    }
}

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
        Some(CompanySignIn {
            oidc: OidcConfig {
                issuer: "https://id.example".to_string(),
                client_id: "company-factory".to_string(),
                client_secret: None,
            },
            allowed_domain: "Example.COM".to_string(),
        }),
        "warp".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap();
    assert_eq!(config.redirect_url(), "http://127.0.0.1:8792/callback");
    assert_eq!(config.allowed_domain.as_deref(), Some("example.com"));
    assert!(!config.cookie_secure);
}

#[test]
fn sign_in_requires_a_company_domain_and_https_sets_a_secure_cookie() {
    let root = tempdir().unwrap();
    let missing = Config::new(
        "127.0.0.1:8792".parse().unwrap(),
        Url::parse("https://factory.aifluxx.co.za").unwrap(),
        secret(),
        false,
        Some(sign_in("")),
        "warp".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap_err();
    assert!(matches!(
        missing,
        ConfigError::Invalid {
            name: "FACTORY_ALLOWED_DOMAIN",
            ..
        }
    ));
    let config = Config::new(
        "127.0.0.1:8792".parse().unwrap(),
        Url::parse("https://factory.aifluxx.co.za").unwrap(),
        secret(),
        false,
        Some(sign_in("aifluxx.co.za")),
        "warp".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap();
    assert!(config.cookie_secure);
}

#[test]
fn a_public_bind_is_rejected() {
    let root = tempdir().unwrap();
    let error = Config::new(
        "0.0.0.0:8792".parse().unwrap(),
        Url::parse("https://factory.aifluxx.co.za").unwrap(),
        secret(),
        true,
        None,
        "warp".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ConfigError::Invalid {
            name: "FACTORY_BIND",
            ..
        }
    ));
}

#[test]
fn google_sign_in_requires_its_own_client_secret() {
    let root = tempdir().unwrap();
    let error = Config::new(
        "127.0.0.1:8792".parse().unwrap(),
        Url::parse("https://factory.aifluxx.co.za").unwrap(),
        secret(),
        true,
        Some(CompanySignIn {
            oidc: OidcConfig {
                issuer: "https://accounts.google.com".to_string(),
                client_id: "company-factory".to_string(),
                client_secret: None,
            },
            allowed_domain: "aifluxx.com".to_string(),
        }),
        "warp".to_string(),
        root.path().to_path_buf(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ConfigError::Invalid {
            name: "OIDC_CLIENT_SECRET",
            ..
        }
    ));
}
