use std::{collections::HashMap, sync::Mutex};

use instant::Instant;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::config::LOGIN_TTL;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub hosted_domain: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Discovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
}

#[derive(Debug)]
pub enum OidcError {
    Discovery,
    Token,
    Untrusted,
}

struct PendingLogin {
    verifier: String,
    created: Instant,
}

pub struct LoginStore {
    inner: Mutex<HashMap<String, PendingLogin>>,
}

impl LoginStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn start(&self, verifier: String) -> String {
        self.prune();
        let state = random_token();
        self.inner.lock().expect("login lock").insert(
            state.clone(),
            PendingLogin {
                verifier,
                created: Instant::now(),
            },
        );
        state
    }

    pub fn take(&self, state: &str) -> Option<String> {
        let pending = self.inner.lock().expect("login lock").remove(state)?;
        if pending.created.elapsed() > LOGIN_TTL {
            None
        } else {
            Some(pending.verifier)
        }
    }

    fn prune(&self) {
        let mut guard = self.inner.lock().expect("login lock");
        guard.retain(|_, pending| pending.created.elapsed() <= LOGIN_TTL);
    }
}

pub fn pkce_pair() -> (String, String) {
    let verifier = random_token();
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);
    (verifier, challenge)
}

pub fn authorization_url(
    discovery: &Discovery,
    client_id: &str,
    redirect_url: &str,
    state: &str,
    challenge: &str,
    hosted_domain: &str,
) -> Result<String, OidcError> {
    let mut url =
        reqwest::Url::parse(&discovery.authorization_endpoint).map_err(|_| OidcError::Discovery)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_url)
        .append_pair("scope", "openid email")
        .append_pair("state", state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("hd", hosted_domain);
    Ok(url.to_string())
}

/// Accept only a verified address in the configured company domain.
///
/// `hd` on the authorization request is only a hint. The token must carry the
/// same hosted domain, and the email domain must match it. A personal Google
/// account has no hosted domain and is refused.
pub fn company_domain_allowed(identity: &Identity, allowed_domain: &str) -> Result<(), OidcError> {
    let Some(hosted) = identity.hosted_domain.as_deref() else {
        return Err(OidcError::Untrusted);
    };
    if !same_domain(hosted, allowed_domain) {
        return Err(OidcError::Untrusted);
    }
    if !identity.email_verified {
        return Err(OidcError::Untrusted);
    }
    let Some(email) = identity.email.as_deref() else {
        return Err(OidcError::Untrusted);
    };
    let Some(domain) = email_domain(email) else {
        return Err(OidcError::Untrusted);
    };
    if !same_domain(domain, allowed_domain) || !same_domain(domain, hosted) {
        return Err(OidcError::Untrusted);
    }
    Ok(())
}

fn same_domain(left: &str, right: &str) -> bool {
    !left.is_empty() && left.eq_ignore_ascii_case(right)
}

fn email_domain(email: &str) -> Option<&str> {
    let (local, domain) = email.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') {
        return None;
    }
    Some(domain)
}

pub async fn discover(http: &reqwest::Client, issuer: &str) -> Result<Discovery, OidcError> {
    let url = format!(
        "{}/.well-known/openid-configuration",
        issuer.trim_end_matches('/')
    );
    let response = http
        .get(url)
        .send()
        .await
        .map_err(|_| OidcError::Discovery)?;
    if !response.status().is_success() {
        return Err(OidcError::Discovery);
    }
    let document: DiscoveryDocument = response.json().await.map_err(|_| OidcError::Discovery)?;
    if document.issuer.trim_end_matches('/') != issuer.trim_end_matches('/') {
        return Err(OidcError::Untrusted);
    }
    Ok(Discovery {
        issuer: document.issuer,
        authorization_endpoint: document.authorization_endpoint,
        token_endpoint: document.token_endpoint,
        jwks_uri: document.jwks_uri,
    })
}

pub async fn exchange_code(
    http: &reqwest::Client,
    token_endpoint: &str,
    code: &str,
    redirect_url: &str,
    client_id: &str,
    client_secret: Option<&str>,
    verifier: &str,
) -> Result<String, OidcError> {
    let mut fields = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("redirect_uri", redirect_url.to_string()),
        ("client_id", client_id.to_string()),
        ("code_verifier", verifier.to_string()),
    ];
    if let Some(secret) = client_secret {
        fields.push(("client_secret", secret.to_string()));
    }
    let response = http
        .post(token_endpoint)
        .form(&fields)
        .send()
        .await
        .map_err(|_| OidcError::Token)?;
    if !response.status().is_success() {
        return Err(OidcError::Token);
    }
    let body: TokenBody = response.json().await.map_err(|_| OidcError::Token)?;
    if body.id_token.is_empty() {
        return Err(OidcError::Untrusted);
    }
    Ok(body.id_token)
}

pub async fn fetch_jwks(http: &reqwest::Client, jwks_uri: &str) -> Result<String, OidcError> {
    let response = http
        .get(jwks_uri)
        .send()
        .await
        .map_err(|_| OidcError::Discovery)?;
    if !response.status().is_success() {
        return Err(OidcError::Discovery);
    }
    response.text().await.map_err(|_| OidcError::Discovery)
}

pub fn verify_id_token(
    token: &str,
    jwks_json: &str,
    issuer: &str,
    audience: &str,
) -> Result<Identity, OidcError> {
    let header = decode_header(token).map_err(|_| OidcError::Untrusted)?;
    if header.alg != Algorithm::RS256 {
        return Err(OidcError::Untrusted);
    }
    let kid = header.kid.ok_or(OidcError::Untrusted)?;
    let jwks: Jwks = serde_json::from_str(jwks_json).map_err(|_| OidcError::Untrusted)?;
    let jwk = jwks
        .keys
        .into_iter()
        .find(|key| key.common.key_id.as_deref() == Some(kid.as_str()))
        .ok_or(OidcError::Untrusted)?;
    let key = DecodingKey::from_jwk(&jwk).map_err(|_| OidcError::Untrusted)?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[audience]);
    validation.set_issuer(&[issuer]);
    let data = decode::<IdClaims>(token, &key, &validation).map_err(|_| OidcError::Untrusted)?;
    if data.claims.sub.is_empty() {
        return Err(OidcError::Untrusted);
    }
    Ok(Identity {
        sub: data.claims.sub,
        email: data.claims.email.filter(|email| !email.is_empty()),
        email_verified: data.claims.email_verified.unwrap_or(false),
        hosted_domain: data.claims.hd.filter(|domain| !domain.is_empty()),
    })
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[derive(Deserialize)]
struct DiscoveryDocument {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
}

#[derive(Deserialize)]
struct TokenBody {
    id_token: String,
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<jsonwebtoken::jwk::Jwk>,
}

#[derive(Deserialize)]
struct IdClaims {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    hd: Option<String>,
}

#[cfg(test)]
#[path = "oidc_tests.rs"]
mod tests;
