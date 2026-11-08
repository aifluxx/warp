use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::{
    byok::{self, ByokError},
    config::SESSION_TTL,
};

type HmacSha256 = Hmac<Sha256>;

pub const COOKIE_NAME: &str = "aifluxx_factory_session";

#[derive(Clone, Debug)]
pub struct SessionView {
    pub id: String,
    pub sub: String,
    pub email: Option<String>,
    pub repo_id: String,
    pub byok_fingerprint: Option<String>,
}

struct Session {
    sub: String,
    email: Option<String>,
    repo_id: String,
    expires_at: u64,
    byok_fingerprint: Option<String>,
    byok: Option<String>,
}

pub struct SessionStore {
    secret: Vec<u8>,
    secure: bool,
    inner: Mutex<HashMap<String, Session>>,
}

impl SessionStore {
    pub fn new(secret: Vec<u8>, secure: bool) -> Self {
        Self {
            secret,
            secure,
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn issue(&self, sub: String, email: Option<String>, repo_id: String) -> String {
        let id = Uuid::new_v4().to_string();
        let expires_at = expiry_unix(SESSION_TTL);
        self.inner.lock().expect("session lock").insert(
            id.clone(),
            Session {
                sub,
                email,
                repo_id,
                expires_at,
                byok_fingerprint: None,
                byok: None,
            },
        );
        self.cookie_header(&id, expires_at, SESSION_TTL.as_secs())
    }

    pub fn read(&self, cookie_header: Option<&str>) -> Option<SessionView> {
        let value = cookie_value(cookie_header?)?;
        let (id, expires_at) = self.verified_cookie(&value)?;
        let now = unix_now();
        if expires_at <= now {
            return None;
        }
        let sessions = self.inner.lock().expect("session lock");
        let session = sessions.get(&id)?;
        if session.expires_at <= now {
            return None;
        }
        Some(SessionView {
            id,
            sub: session.sub.clone(),
            email: session.email.clone(),
            repo_id: session.repo_id.clone(),
            byok_fingerprint: session.byok_fingerprint.clone(),
        })
    }

    pub fn set_byok(&self, session_id: &str, key: &str) -> Result<String, ByokError> {
        let fingerprint = byok::fingerprint(key)?;
        let mut sessions = self.inner.lock().expect("session lock");
        let session = sessions.get_mut(session_id).ok_or(ByokError::Empty)?;
        session.byok = Some(key.to_string());
        session.byok_fingerprint = Some(fingerprint.clone());
        Ok(fingerprint)
    }

    pub fn has_byok(&self, session_id: &str) -> bool {
        self.inner
            .lock()
            .expect("session lock")
            .get(session_id)
            .and_then(|session| session.byok.as_ref())
            .is_some()
    }

    pub fn logout(&self, cookie_header: Option<&str>) -> String {
        if let Some(value) = cookie_header.and_then(cookie_value)
            && let Some((id, _)) = self.verified_cookie(&value)
        {
            let mut sessions = self.inner.lock().expect("session lock");
            if let Some(session) = sessions.get_mut(&id) {
                session.byok = None;
                session.byok_fingerprint = None;
            }
            sessions.remove(&id);
        }
        clear_cookie(self.secure)
    }

    fn cookie_header(&self, id: &str, expires_at: u64, max_age: u64) -> String {
        let value = self.sign(id, expires_at);
        let secure = if self.secure { "; Secure" } else { "" };
        format!("{COOKIE_NAME}={value}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age}{secure}")
    }

    fn sign(&self, id: &str, expires_at: u64) -> String {
        let payload = format!("{id}.{expires_at}");
        let mac = mac_bytes(&self.secret, payload.as_bytes());
        format!("{payload}.{}", URL_SAFE_NO_PAD.encode(mac))
    }

    fn verified_cookie(&self, value: &str) -> Option<(String, u64)> {
        let (payload, signature) = value.rsplit_once('.')?;
        let (id, expires) = payload.rsplit_once('.')?;
        let expires_at: u64 = expires.parse().ok()?;
        let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
        let mut mac = HmacSha256::new_from_slice(&self.secret).ok()?;
        mac.update(payload.as_bytes());
        mac.verify_slice(&signature).ok()?;
        Some((id.to_string(), expires_at))
    }
}

fn mac_bytes(secret: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac key");
    mac.update(payload);
    mac.finalize().into_bytes().to_vec()
}

fn cookie_value(header: &str) -> Option<String> {
    header.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == COOKIE_NAME).then(|| value.to_string())
    })
}

fn clear_cookie(secure: bool) -> String {
    let secure = if secure { "; Secure" } else { "" };
    format!("{COOKIE_NAME}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{secure}")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

fn expiry_unix(ttl: Duration) -> u64 {
    unix_now().saturating_add(ttl.as_secs())
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
