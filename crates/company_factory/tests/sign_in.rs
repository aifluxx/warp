//! HTTP coverage for company sign-in.
//!
//! The identity provider in this file is a fixture. It signs RS256 tokens so
//! the crate's normal verifier runs. It is not a production login and it is
//! not Warp's account service.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::{Form, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use company_factory::{Config, OidcConfig};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPrivateKey, traits::PublicKeyParts};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

const CLIENT_ID: &str = "company-factory-test";
const CLIENT_SECRET: &str = "fixture-client-secret-not-a-platform-key";
const ALICE_KEY: &str = "alice-personal-model-key";
const PLATFORM_KEY: &str = "sk-platform-must-not-be-used";

struct Issued {
    sub: String,
    email: String,
    challenge: String,
}

struct FixtureState {
    issuer: String,
    pem: String,
    jwks: Value,
    codes: Mutex<HashMap<String, Issued>>,
}

#[derive(Clone)]
struct Fixture {
    issuer: String,
}

async fn start_fixture() -> Fixture {
    let mut rng = rand::thread_rng();
    let key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let pem = key
        .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
        .unwrap()
        .to_string();
    let jwks = json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": "fixture-1",
            "n": URL_SAFE_NO_PAD.encode(key.n().to_bytes_be()),
            "e": URL_SAFE_NO_PAD.encode(key.e().to_bytes_be()),
        }]
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}", listener.local_addr().unwrap());
    let state = Arc::new(FixtureState {
        issuer: issuer.clone(),
        pem,
        jwks,
        codes: Mutex::new(HashMap::new()),
    });
    let app = Router::new()
        .route("/.well-known/openid-configuration", get(discovery))
        .route("/authorize", get(authorize))
        .route("/token", post(token))
        .route("/jwks", get(jwks_document))
        .with_state(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    Fixture { issuer }
}

async fn discovery(State(state): State<Arc<FixtureState>>) -> Json<Value> {
    Json(json!({
        "issuer": state.issuer,
        "authorization_endpoint": format!("{}/authorize", state.issuer),
        "token_endpoint": format!("{}/token", state.issuer),
        "jwks_uri": format!("{}/jwks", state.issuer),
    }))
}

async fn jwks_document(State(state): State<Arc<FixtureState>>) -> Json<Value> {
    Json(state.jwks.clone())
}

#[derive(serde::Deserialize)]
struct AuthorizeQuery {
    client_id: String,
    redirect_uri: String,
    state: String,
    code_challenge: String,
    login_hint: Option<String>,
    email: Option<String>,
}

async fn authorize(
    State(state): State<Arc<FixtureState>>,
    Query(query): Query<AuthorizeQuery>,
) -> Response {
    if query.client_id != CLIENT_ID {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let sub = query
        .login_hint
        .unwrap_or_else(|| "fixture-alice".to_string());
    let email = query
        .email
        .unwrap_or_else(|| format!("{sub}@fixture.example"));
    let code = format!("code-{sub}-{}", state.codes.lock().unwrap().len());
    state.codes.lock().unwrap().insert(
        code.clone(),
        Issued {
            sub,
            email,
            challenge: query.code_challenge,
        },
    );
    let mut redirect = reqwest::Url::parse(&query.redirect_uri).unwrap();
    redirect
        .query_pairs_mut()
        .append_pair("code", &code)
        .append_pair("state", &query.state);
    Redirect::temporary(redirect.as_str()).into_response()
}

#[derive(serde::Deserialize)]
struct TokenForm {
    code: String,
    code_verifier: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

async fn token(State(state): State<Arc<FixtureState>>, Form(form): Form<TokenForm>) -> Response {
    if form.client_id != CLIENT_ID || form.client_secret != CLIENT_SECRET {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !form.redirect_uri.ends_with("/callback") {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let Some(issued) = state.codes.lock().unwrap().remove(&form.code) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let digest = Sha256::digest(form.code_verifier.as_bytes());
    if URL_SAFE_NO_PAD.encode(digest) != issued.challenge {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = TokenClaims {
        sub: issued.sub,
        email: issued.email,
        iss: state.issuer.clone(),
        aud: CLIENT_ID.to_string(),
        iat: now,
        exp: now + 600,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("fixture-1".to_string());
    let id_token = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(state.pem.as_bytes()).unwrap(),
    )
    .unwrap();
    Json(json!({
        "token_type": "Bearer",
        "expires_in": 600,
        "id_token": id_token,
    }))
    .into_response()
}

#[derive(Serialize)]
struct TokenClaims {
    sub: String,
    email: String,
    iss: String,
    aud: String,
    iat: u64,
    exp: u64,
}

struct Harness {
    addr: String,
    client: reqwest::Client,
    server: company_factory::Server,
}

async fn start(root: &std::path::Path, oidc_issuer: Option<&str>) -> Harness {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);
    let public = format!("http://{addr}");
    let oidc = oidc_issuer.map(|issuer| OidcConfig {
        issuer: issuer.to_string(),
        client_id: CLIENT_ID.to_string(),
        client_secret: Some(CLIENT_SECRET.to_string()),
    });
    let config = Config::new(
        addr,
        reqwest::Url::parse(&public).unwrap(),
        b"test-session-secret-is-32-bytes!".to_vec(),
        false,
        oidc,
        "warp".to_string(),
        root.to_path_buf(),
    )
    .unwrap();
    let server = company_factory::serve(config).await.unwrap();
    Harness {
        addr: public,
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        server,
    }
}

async fn sign_in(harness: &Harness, login_hint: &str, email: &str) -> String {
    let login = harness
        .client
        .get(format!("{}/login", harness.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::TEMPORARY_REDIRECT);
    let mut authorize =
        reqwest::Url::parse(login.headers().get("location").unwrap().to_str().unwrap()).unwrap();
    authorize
        .query_pairs_mut()
        .append_pair("login_hint", login_hint)
        .append_pair("email", email);
    let consented = harness.client.get(authorize).send().await.unwrap();
    assert_eq!(consented.status(), StatusCode::TEMPORARY_REDIRECT);
    let callback = consented
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    let established = harness.client.get(callback).send().await.unwrap();
    assert_eq!(established.status(), StatusCode::TEMPORARY_REDIRECT);
    established
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn unsigned_requests_are_refused_when_sign_in_is_not_configured() {
    let root = tempdir().unwrap();
    let harness = start(root.path(), None).await;
    let workspace = harness
        .client
        .get(format!("{}/workspace", harness.addr))
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(workspace.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        workspace.json::<Value>().await.unwrap()["error"],
        "unauthenticated"
    );

    let login = harness
        .client
        .get(format!("{}/login", harness.addr))
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = login.json::<Value>().await.unwrap();
    assert_eq!(body["error"], "company_sign_in_not_configured");
    assert!(
        body["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "OIDC_ISSUER")
    );

    let logout = harness
        .client
        .get(format!("{}/logout", harness.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::METHOD_NOT_ALLOWED);
    harness.server.shutdown().await;
}

#[tokio::test]
async fn fixture_provider_sign_in_is_isolated_and_distinct_from_warp() {
    // Fixture evidence only. This is not a parent-computer sign-in.
    let root = tempdir().unwrap();
    std::fs::write(root.path().join("notes.txt"), "visible").unwrap();
    std::fs::write(root.path().join(".secret"), "hidden").unwrap();
    let fixture = start_fixture().await;
    let harness = start(root.path(), Some(&fixture.issuer)).await;

    let home = harness
        .client
        .get(&harness.addr)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(home.contains("Not Warp Factories"));
    assert!(home.contains("Not the Warp contributions dashboard"));
    assert!(!home.contains("ttyd"));

    let alice = sign_in(&harness, "fixture-alice", "a<b>@example.com").await;
    let page = harness
        .client
        .get(format!("{}/dashboard", harness.addr))
        .header("cookie", &alice)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(page.contains("AIFluxx company factory"));
    assert!(page.contains("Not Warp Factories"));
    assert!(page.contains("a&lt;b&gt;@example.com"));
    assert!(!page.contains("<b>@example.com"));
    assert!(page.contains("Warp's hosted agent and factories are not signed in here"));

    let locked = harness
        .client
        .post(format!("{}/workspace/action", harness.addr))
        .header("cookie", &alice)
        .header("accept", "application/json")
        .form(&[("repo_id", "warp")])
        .send()
        .await
        .unwrap();
    assert_eq!(locked.status(), StatusCode::FORBIDDEN);
    let locked_body = locked.text().await.unwrap();
    assert!(locked_body.contains("byok_required"));
    assert!(!locked_body.contains(PLATFORM_KEY));
    assert!(!locked_body.contains(ALICE_KEY));

    let saved = harness
        .client
        .post(format!("{}/byok", harness.addr))
        .header("cookie", &alice)
        .header("accept", "application/json")
        .form(&[("api_key", ALICE_KEY)])
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), StatusCode::OK);
    let saved_body = saved.text().await.unwrap();
    assert!(!saved_body.contains(ALICE_KEY));
    let fingerprint = serde_json::from_str::<Value>(&saved_body).unwrap()["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    let browser_save = harness
        .client
        .post(format!("{}/byok", harness.addr))
        .header("cookie", &alice)
        .form(&[("api_key", ALICE_KEY)])
        .send()
        .await
        .unwrap();
    assert_eq!(browser_save.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        browser_save.headers().get("location").unwrap(),
        "/dashboard"
    );

    let other_repo = harness
        .client
        .post(format!("{}/workspace/action", harness.addr))
        .header("cookie", &alice)
        .header("accept", "application/json")
        .form(&[("repo_id", "../other")])
        .send()
        .await
        .unwrap();
    assert_eq!(other_repo.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        other_repo.json::<Value>().await.unwrap()["error"],
        "repository_not_assigned"
    );

    let inventory = harness
        .client
        .post(format!("{}/workspace/action", harness.addr))
        .header("cookie", &alice)
        .header("accept", "application/json")
        .form(&[("repo_id", "warp")])
        .send()
        .await
        .unwrap();
    assert_eq!(inventory.status(), StatusCode::OK);
    let inventory = inventory.json::<Value>().await.unwrap();
    assert_eq!(inventory["names"], json!(["notes.txt"]));

    let bob = sign_in(&harness, "fixture-bob", "bob@example.com").await;
    let bob_key = harness
        .client
        .get(format!("{}/byok", harness.addr))
        .header("cookie", &bob)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(bob_key["configured"], false);
    assert!(bob_key["fingerprint"].is_null());
    let bob_action = harness
        .client
        .post(format!("{}/workspace/action", harness.addr))
        .header("cookie", &bob)
        .header("accept", "application/json")
        .form(&[("repo_id", "warp")])
        .send()
        .await
        .unwrap();
    assert_eq!(bob_action.status(), StatusCode::FORBIDDEN);

    let alice_key = harness
        .client
        .get(format!("{}/byok", harness.addr))
        .header("cookie", &alice)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(alice_key["fingerprint"], fingerprint);

    let signed_out = harness
        .client
        .post(format!("{}/logout", harness.addr))
        .header("cookie", &alice)
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(signed_out.status(), StatusCode::OK);
    let again = harness
        .client
        .get(format!("{}/dashboard", harness.addr))
        .header("cookie", &alice)
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(again.status(), StatusCode::UNAUTHORIZED);

    let tampered = alice.replace("aifluxx_factory_session=", "aifluxx_factory_session=x");
    let forged = harness
        .client
        .get(format!("{}/dashboard", harness.addr))
        .header("cookie", tampered)
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);
    harness.server.shutdown().await;
}
