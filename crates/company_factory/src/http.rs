use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;
use tokio::{sync::oneshot, task::JoinHandle};

use crate::{
    byok::ByokError,
    config::Config,
    oidc::{self, LoginStore},
    repo::{self, RepoError},
    session::SessionStore,
};

pub struct Server {
    pub addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl Server {
    pub async fn shutdown(mut self) {
        if let Some(sender) = self.shutdown_tx.take() {
            let _ = sender.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

pub struct AppState {
    config: Config,
    sessions: SessionStore,
    logins: LoginStore,
    http: reqwest::Client,
    discovery: tokio::sync::Mutex<Option<oidc::Discovery>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let sessions = SessionStore::new(config.session_secret.clone(), config.cookie_secure);
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest client");
        Self {
            config,
            sessions,
            logins: LoginStore::new(),
            http,
            discovery: tokio::sync::Mutex::new(None),
        }
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", post(logout))
        .route("/dashboard", get(dashboard))
        .route("/workspace", get(workspace))
        .route("/byok", get(byok_status).post(set_byok))
        .route("/workspace/action", post(workspace_action))
        .with_state(state)
}

pub async fn serve(config: Config) -> Result<Server, std::io::Error> {
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    let addr = listener.local_addr()?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let app = router(Arc::new(AppState::new(config)));
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });
    Ok(Server {
        addr,
        shutdown_tx: Some(shutdown_tx),
        task: Some(task),
    })
}

async fn home(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let signed_in = state.sessions.read(cookie(&headers)).is_some();
    if wants_json(&headers) {
        return Json(json!({
            "service": "aifluxx-company-factory",
            "product": "AIFluxx company factory",
            "not": ["Warp Factories", "Warp contributions dashboard"],
            "authenticated": signed_in,
        }))
        .into_response();
    }
    html(
        StatusCode::OK,
        None,
        page(
            "AIFluxx company factory",
            "<p><a href=\"/login\">Sign in</a></p><p><a href=\"/dashboard\">Workspace entry</a></p>",
        ),
    )
}

async fn login(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let Some(oidc) = state.config.oidc.clone() else {
        return not_configured(&headers);
    };
    let discovery = match load_discovery(&state, &oidc.issuer).await {
        Ok(discovery) => discovery,
        Err(_) => return bad_identity(&headers),
    };
    let (verifier, challenge) = oidc::pkce_pair();
    let csrf = state.logins.start(verifier);
    let redirect = match oidc::authorization_url(
        &discovery,
        &oidc.client_id,
        &state.config.redirect_url(),
        &csrf,
        &challenge,
    ) {
        Ok(redirect) => redirect,
        Err(_) => return bad_identity(&headers),
    };
    Redirect::temporary(&redirect).into_response()
}

async fn callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let Some(oidc_config) = state.config.oidc.clone() else {
        return not_configured(&headers);
    };
    let (Some(code), Some(csrf)) = (query.code, query.state) else {
        return sign_in_refused(&headers);
    };
    let Some(verifier) = state.logins.take(&csrf) else {
        return sign_in_refused(&headers);
    };
    let discovery = match load_discovery(&state, &oidc_config.issuer).await {
        Ok(discovery) => discovery,
        Err(_) => return sign_in_refused(&headers),
    };
    let token = match oidc::exchange_code(
        &state.http,
        &discovery.token_endpoint,
        &code,
        &state.config.redirect_url(),
        &oidc_config.client_id,
        oidc_config.client_secret.as_deref(),
        &verifier,
    )
    .await
    {
        Ok(token) => token,
        Err(_) => return sign_in_refused(&headers),
    };
    let jwks = match oidc::fetch_jwks(&state.http, &discovery.jwks_uri).await {
        Ok(jwks) => jwks,
        Err(_) => return sign_in_refused(&headers),
    };
    let identity =
        match oidc::verify_id_token(&token, &jwks, &discovery.issuer, &oidc_config.client_id) {
            Ok(identity) => identity,
            Err(_) => return sign_in_refused(&headers),
        };
    let cookie = state
        .sessions
        .issue(identity.sub, identity.email, state.config.repo_id.clone());
    let mut response = Redirect::temporary("/dashboard").into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.parse().expect("cookie"));
    response
}

async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let cookie = state.sessions.logout(cookie(&headers));
    if wants_json(&headers) {
        return (
            StatusCode::OK,
            [(header::SET_COOKIE, cookie)],
            Json(json!({"signed_out": true})),
        )
            .into_response();
    }
    html(
        StatusCode::OK,
        Some(cookie),
        page(
            "Signed out",
            "<p>Signed out of the AIFluxx company factory.</p><p><a href=\"/login\">Sign in</a></p>",
        ),
    )
}

async fn dashboard(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let Some(session) = state.sessions.read(cookie(&headers)) else {
        return unauthenticated(&headers);
    };
    if wants_json(&headers) {
        return Json(json!({
            "service": "aifluxx-company-factory",
            "sub": session.sub,
            "email": session.email,
            "repo_id": session.repo_id,
            "byok_configured": session.byok_fingerprint.is_some(),
        }))
        .into_response();
    }
    let who = escape(session.email.as_deref().unwrap_or(&session.sub));
    let repo = escape(&session.repo_id);
    let key_state = if session.byok_fingerprint.is_some() {
        "Personal model key is set for this sign-in."
    } else {
        "No personal model key is set. The repository action stays refused."
    };
    let body = format!(
        "<p>Signed in as {who}.</p>\
         <p>Assigned repository: {repo}.</p>\
         <p>{key_state}</p>\
         <p>Warp's hosted agent and factories are not signed in here. A Warp API key is not accepted as company sign-in.</p>\
         <form method=\"post\" action=\"/byok\"><label>Your model key <input name=\"api_key\" type=\"password\" autocomplete=\"off\"></label> <button type=\"submit\">Save key</button></form>\
         <form method=\"post\" action=\"/workspace/action\"><input type=\"hidden\" name=\"repo_id\" value=\"{repo}\"><button type=\"submit\">Run repository inventory</button></form>\
         <form method=\"post\" action=\"/logout\"><button type=\"submit\">Sign out</button></form>"
    );
    html(StatusCode::OK, None, page("Workspace entry", &body))
}

async fn workspace(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let Some(session) = state.sessions.read(cookie(&headers)) else {
        return unauthenticated(&headers);
    };
    if wants_json(&headers) {
        return Json(json!({
            "repo_id": session.repo_id,
            "byok_configured": session.byok_fingerprint.is_some(),
        }))
        .into_response();
    }
    Redirect::temporary("/dashboard").into_response()
}

async fn byok_status(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let Some(session) = state.sessions.read(cookie(&headers)) else {
        return unauthenticated(&headers);
    };
    Json(json!({
        "configured": session.byok_fingerprint.is_some(),
        "fingerprint": session.byok_fingerprint,
    }))
    .into_response()
}

async fn set_byok(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<ByokForm>,
) -> Response {
    let Some(session) = state.sessions.read(cookie(&headers)) else {
        return unauthenticated(&headers);
    };
    match state.sessions.set_byok(&session.id, &form.api_key) {
        Ok(fingerprint) => {
            if wants_json(&headers) {
                Json(json!({"configured": true, "fingerprint": fingerprint})).into_response()
            } else {
                Redirect::temporary("/dashboard").into_response()
            }
        }
        Err(ByokError::TooShort | ByokError::Empty | ByokError::TooLong) => refused(
            &headers,
            StatusCode::BAD_REQUEST,
            "personal_key_rejected",
            "That personal model key was rejected. No platform key is used instead.",
        ),
    }
}

async fn workspace_action(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<RepoForm>,
) -> Response {
    let Some(session) = state.sessions.read(cookie(&headers)) else {
        return unauthenticated(&headers);
    };
    if repo::require_assigned(&form.repo_id, &state.config.repo_id).is_err() {
        return refused(
            &headers,
            StatusCode::FORBIDDEN,
            "repository_not_assigned",
            "That repository is not assigned to this slice.",
        );
    }
    if !state.sessions.has_byok(&session.id) {
        return refused(
            &headers,
            StatusCode::FORBIDDEN,
            "byok_required",
            "A personal model key is required. No platform key is used instead.",
        );
    }
    match repo::visible_names(&state.config.repo_root) {
        Ok(names) => {
            if wants_json(&headers) {
                Json(json!({"repo_id": state.config.repo_id, "names": names})).into_response()
            } else {
                let items = names
                    .iter()
                    .map(|name| format!("<li>{}</li>", escape(name)))
                    .collect::<String>();
                html(
                    StatusCode::OK,
                    None,
                    page(
                        "Repository inventory",
                        &format!("<p>Assigned repository inventory.</p><ul>{items}</ul>"),
                    ),
                )
            }
        }
        Err(RepoError::Unreadable | RepoError::NotAssigned) => refused(
            &headers,
            StatusCode::FORBIDDEN,
            "repository_not_assigned",
            "That repository is not assigned to this slice.",
        ),
    }
}

async fn load_discovery(
    state: &AppState,
    issuer: &str,
) -> Result<oidc::Discovery, oidc::OidcError> {
    let mut cached = state.discovery.lock().await;
    if let Some(discovery) = cached.clone() {
        return Ok(discovery);
    }
    let discovery = oidc::discover(&state.http, issuer).await?;
    *cached = Some(discovery.clone());
    Ok(discovery)
}

fn cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
}

fn wants_json(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("application/json"))
}

fn not_configured(headers: &HeaderMap) -> Response {
    let missing = ["OIDC_ISSUER", "OIDC_CLIENT_ID"];
    if wants_json(headers) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "company_sign_in_not_configured",
                "missing": missing,
            })),
        )
            .into_response();
    }
    html(
        StatusCode::SERVICE_UNAVAILABLE,
        None,
        page(
            "Sign-in is not configured",
            "<p>Company sign-in needs OIDC_ISSUER and OIDC_CLIENT_ID from the operator. No account was created.</p>",
        ),
    )
}

fn bad_identity(headers: &HeaderMap) -> Response {
    refused(
        headers,
        StatusCode::BAD_GATEWAY,
        "identity_service_unreachable",
        "The company identity service did not answer. Sign-in was not completed.",
    )
}

fn sign_in_refused(headers: &HeaderMap) -> Response {
    refused(
        headers,
        StatusCode::UNAUTHORIZED,
        "sign_in_refused",
        "Sign-in was not accepted.",
    )
}

fn unauthenticated(headers: &HeaderMap) -> Response {
    refused(
        headers,
        StatusCode::UNAUTHORIZED,
        "unauthenticated",
        "Sign in to the AIFluxx company factory before opening the workspace.",
    )
}

fn refused(headers: &HeaderMap, status: StatusCode, code: &str, message: &str) -> Response {
    if wants_json(headers) {
        (status, Json(json!({"error": code}))).into_response()
    } else {
        html(
            status,
            None,
            page(
                "Refused",
                &format!(
                    "<p>{}</p><p><a href=\"/login\">Sign in</a></p>",
                    escape(message)
                ),
            ),
        )
    }
}

fn html(status: StatusCode, cookie: Option<String>, body: String) -> Response {
    let mut response = (status, security_headers(), Html(body)).into_response();
    if let Some(cookie) = cookie
        && let Ok(value) = cookie.parse()
    {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    response
}

fn security_headers() -> [(header::HeaderName, &'static str); 4] {
    [
        (header::CACHE_CONTROL, "no-store"),
        (
            header::HeaderName::from_static("referrer-policy"),
            "no-referrer",
        ),
        (
            header::HeaderName::from_static("x-content-type-options"),
            "nosniff",
        ),
        (header::HeaderName::from_static("x-frame-options"), "DENY"),
    ]
}

fn page(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>{title}</title></head>\
         <body><header><p><strong>AIFluxx company factory</strong></p>\
         <p>Company development slice. Not Warp Factories. Not the Warp contributions dashboard.</p></header>\
         {body}</body></html>"
    )
}

fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct ByokForm {
    api_key: String,
}

#[derive(Deserialize)]
struct RepoForm {
    repo_id: String,
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
