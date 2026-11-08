use sha2::{Digest, Sha256};

pub const MIN_KEY_LEN: usize = 20;
pub const MAX_KEY_LEN: usize = 4096;

#[derive(Debug, PartialEq, Eq)]
pub enum ByokError {
    Empty,
    TooShort,
    TooLong,
}

/// Fingerprint a personal key. The key itself is not returned.
pub fn fingerprint(key: &str) -> Result<String, ByokError> {
    if key.trim() != key || key.is_empty() {
        return Err(ByokError::Empty);
    }
    if key.len() < MIN_KEY_LEN {
        return Err(ByokError::TooShort);
    }
    if key.len() > MAX_KEY_LEN {
        return Err(ByokError::TooLong);
    }
    let digest = Sha256::digest(key.as_bytes());
    Ok(hex::encode(digest)[..12].to_string())
}

#[cfg(test)]
#[path = "byok_tests.rs"]
mod tests;
