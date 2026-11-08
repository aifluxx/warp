use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use super::verify_id_token;

#[test]
fn an_unsigned_token_is_not_accepted() {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let claims =
        URL_SAFE_NO_PAD.encode(r#"{"sub":"person","iss":"https://id.example","aud":"client"}"#);
    let token = format!("{header}.{claims}.");
    let error = verify_id_token(&token, r#"{"keys":[]}"#, "https://id.example", "client");
    assert!(error.is_err());
}
