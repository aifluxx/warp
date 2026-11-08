use std::{net::SocketAddr, path::PathBuf, time::Duration};

use thiserror::Error;
use url::Url;

pub const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);
pub const LOGIN_TTL: Duration = Duration::from_secs(10 * 60);

/// Identity-provider settings supplied by the operator. None are built in.
#[derive(Clone)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

/// Company sign-in. The domain is required whenever the provider is set.
#[derive(Clone)]
pub struct CompanySignIn {
    pub oidc: OidcConfig,
    pub allowed_domain: String,
}

/// Runtime settings. Secrets are omitted from debug output.
#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub public_url: Url,
    pub session_secret: Vec<u8>,
    pub cookie_secure: bool,
    pub oidc: Option<OidcConfig>,
    pub allowed_domain: Option<String>,
    pub repo_id: String,
    pub repo_root: PathBuf,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Config")
            .field("bind", &self.bind)
            .field("public_url", &self.public_url)
            .field("cookie_secure", &self.cookie_secure)
            .field("oidc_configured", &self.oidc.is_some())
            .field("allowed_domain", &self.allowed_domain)
            .field("repo_id", &self.repo_id)
            .field("repo_root", &self.repo_root)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing {missing:?}")]
    Incomplete { missing: Vec<&'static str> },
    #[error("{name} is invalid: {message}")]
    Invalid { name: &'static str, message: String },
}

impl Config {
    pub fn new(
        bind: SocketAddr,
        public_url: Url,
        session_secret: Vec<u8>,
        cookie_secure: bool,
        sign_in: Option<CompanySignIn>,
        repo_id: String,
        repo_root: PathBuf,
    ) -> Result<Self, ConfigError> {
        validate_secret(&session_secret)?;
        validate_repo_id(&repo_id)?;
        if !repo_root.is_dir() {
            return Err(ConfigError::Invalid {
                name: "FACTORY_REPO_ROOT",
                message: "directory does not exist".to_string(),
            });
        }
        validate_bind(bind)?;
        let (oidc, allowed_domain) = match sign_in {
            Some(sign_in) => {
                validate_oidc(&sign_in.oidc)?;
                let domain = validate_domain(&sign_in.allowed_domain)?;
                (Some(sign_in.oidc), Some(domain))
            }
            None => (None, None),
        };
        let mut public_url = public_url;
        public_url.set_query(None);
        public_url.set_fragment(None);
        let public_url =
            Url::parse(public_url.as_str().trim_end_matches('/')).map_err(|error| {
                ConfigError::Invalid {
                    name: "FACTORY_PUBLIC_URL",
                    message: error.to_string(),
                }
            })?;
        let cookie_secure = cookie_secure || public_url.scheme() == "https";
        Ok(Self {
            bind,
            public_url,
            session_secret,
            cookie_secure,
            oidc,
            allowed_domain,
            repo_id,
            repo_root,
        })
    }

    pub fn redirect_url(&self) -> String {
        format!(
            "{}/callback",
            self.public_url.as_str().trim_end_matches('/')
        )
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let mut missing = Vec::new();
        let secret = match std::env::var("FACTORY_SESSION_SECRET") {
            Ok(value) if !value.is_empty() => value.into_bytes(),
            _ => {
                missing.push("FACTORY_SESSION_SECRET");
                Vec::new()
            }
        };
        let public_url = match std::env::var("FACTORY_PUBLIC_URL") {
            Ok(value) if !value.is_empty() => value,
            _ => {
                missing.push("FACTORY_PUBLIC_URL");
                String::new()
            }
        };
        let repo_id = match std::env::var("FACTORY_REPO_ID") {
            Ok(value) if !value.is_empty() => value,
            _ => {
                missing.push("FACTORY_REPO_ID");
                String::new()
            }
        };
        let repo_root = match std::env::var("FACTORY_REPO_ROOT") {
            Ok(value) if !value.is_empty() => value,
            _ => {
                missing.push("FACTORY_REPO_ROOT");
                String::new()
            }
        };
        if !missing.is_empty() {
            return Err(ConfigError::Incomplete { missing });
        }
        let bind = std::env::var("FACTORY_BIND")
            .unwrap_or_else(|_| "127.0.0.1:8792".to_string())
            .parse()
            .map_err(|error| ConfigError::Invalid {
                name: "FACTORY_BIND",
                message: format!("{error}"),
            })?;
        let public_url = Url::parse(&public_url).map_err(|error| ConfigError::Invalid {
            name: "FACTORY_PUBLIC_URL",
            message: error.to_string(),
        })?;
        let cookie_secure = matches!(
            std::env::var("FACTORY_COOKIE_SECURE").ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE")
        );
        let issuer = std::env::var("OIDC_ISSUER")
            .ok()
            .filter(|value| !value.is_empty());
        let client_id = std::env::var("OIDC_CLIENT_ID")
            .ok()
            .filter(|value| !value.is_empty());
        let oidc = match (issuer, client_id) {
            (Some(issuer), Some(client_id)) => {
                let client_secret = std::env::var("OIDC_CLIENT_SECRET")
                    .ok()
                    .filter(|value| !value.is_empty());
                Some(OidcConfig {
                    issuer,
                    client_id,
                    client_secret,
                })
            }
            (None, None) => None,
            _ => {
                return Err(ConfigError::Incomplete {
                    missing: vec!["OIDC_ISSUER", "OIDC_CLIENT_ID"],
                });
            }
        };
        let allowed_domain = std::env::var("FACTORY_ALLOWED_DOMAIN")
            .ok()
            .filter(|value| !value.is_empty());
        let sign_in = match (oidc, allowed_domain) {
            (Some(oidc), Some(allowed_domain)) => Some(CompanySignIn {
                oidc,
                allowed_domain,
            }),
            (Some(_), None) => {
                return Err(ConfigError::Incomplete {
                    missing: vec!["FACTORY_ALLOWED_DOMAIN"],
                });
            }
            (None, _) => None,
        };
        Self::new(
            bind,
            public_url,
            secret,
            cookie_secure,
            sign_in,
            repo_id,
            PathBuf::from(repo_root),
        )
    }
}

fn validate_secret(secret: &[u8]) -> Result<(), ConfigError> {
    if secret.len() < 32 {
        return Err(ConfigError::Invalid {
            name: "FACTORY_SESSION_SECRET",
            message: "must be at least 32 bytes".to_string(),
        });
    }
    Ok(())
}

fn validate_repo_id(repo_id: &str) -> Result<(), ConfigError> {
    let plain = !repo_id.is_empty()
        && repo_id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        });
    if !plain {
        return Err(ConfigError::Invalid {
            name: "FACTORY_REPO_ID",
            message: "use a single repository name without a path".to_string(),
        });
    }
    Ok(())
}

fn validate_domain(domain: &str) -> Result<String, ConfigError> {
    let domain = domain.trim().to_ascii_lowercase();
    let plain = domain.len() >= 3
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
        && domain.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '.' || character == '-'
        });
    if !plain {
        return Err(ConfigError::Invalid {
            name: "FACTORY_ALLOWED_DOMAIN",
            message: "use the company hosted domain, such as aifluxx.com".to_string(),
        });
    }
    Ok(domain)
}

fn validate_oidc(oidc: &OidcConfig) -> Result<(), ConfigError> {
    let issuer = Url::parse(&oidc.issuer).map_err(|error| ConfigError::Invalid {
        name: "OIDC_ISSUER",
        message: error.to_string(),
    })?;
    if issuer.scheme() != "https" && issuer.scheme() != "http" {
        return Err(ConfigError::Invalid {
            name: "OIDC_ISSUER",
            message: "issuer must be an http or https URL".to_string(),
        });
    }
    if oidc.client_id.is_empty() {
        return Err(ConfigError::Invalid {
            name: "OIDC_CLIENT_ID",
            message: "client id is empty".to_string(),
        });
    }
    if google_issuer(&issuer)
        && oidc
            .client_secret
            .as_deref()
            .is_none_or(|secret| secret.is_empty())
    {
        return Err(ConfigError::Invalid {
            name: "OIDC_CLIENT_SECRET",
            message: "a dedicated Google web client requires its own secret".to_string(),
        });
    }
    Ok(())
}

fn validate_bind(bind: SocketAddr) -> Result<(), ConfigError> {
    if bind.ip().is_loopback() {
        return Ok(());
    }
    Err(ConfigError::Invalid {
        name: "FACTORY_BIND",
        message: "listen on 127.0.0.1 only; the reverse proxy is the public path".to_string(),
    })
}

fn google_issuer(issuer: &Url) -> bool {
    issuer
        .host_str()
        .is_some_and(|host| host.eq_ignore_ascii_case("accounts.google.com"))
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
