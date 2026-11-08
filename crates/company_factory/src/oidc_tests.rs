use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use super::{Identity, company_domain_allowed, verify_id_token};

#[test]
fn an_unsigned_token_is_not_accepted() {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let claims =
        URL_SAFE_NO_PAD.encode(r#"{"sub":"person","iss":"https://id.example","aud":"client"}"#);
    let token = format!("{header}.{claims}.");
    let error = verify_id_token(&token, r#"{"keys":[]}"#, "https://id.example", "client");
    assert!(error.is_err());
}

fn person(email: Option<&str>, hosted: Option<&str>, verified: bool) -> Identity {
    Identity {
        sub: "person".to_string(),
        email: email.map(str::to_string),
        email_verified: verified,
        hosted_domain: hosted.map(str::to_string),
    }
}

#[test]
fn only_the_company_domain_is_accepted() {
    assert!(
        company_domain_allowed(
            &person(Some("a@aifluxx.com"), Some("aifluxx.com"), true),
            "aifluxx.com",
        )
        .is_ok()
    );
    assert!(
        company_domain_allowed(
            &person(Some("a@aifluxx.co.za"), Some("aifluxx.co.za"), true),
            "aifluxx.com",
        )
        .is_err()
    );
    assert!(
        company_domain_allowed(
            &person(Some("a@aifluxx.com"), Some("aifluxx.co.za"), true),
            "aifluxx.com",
        )
        .is_err()
    );
    assert!(
        company_domain_allowed(&person(Some("person@gmail.com"), None, true), "aifluxx.com",)
            .is_err()
    );
    assert!(
        company_domain_allowed(
            &person(Some("person@gmail.com"), Some("gmail.com"), true),
            "aifluxx.com",
        )
        .is_err()
    );
    assert!(
        company_domain_allowed(
            &person(Some("a@evil.aifluxx.com"), Some("aifluxx.com"), true),
            "aifluxx.com",
        )
        .is_err()
    );
    assert!(
        company_domain_allowed(
            &person(Some("a@aifluxx.com.evil.com"), Some("aifluxx.com"), true),
            "aifluxx.com",
        )
        .is_err()
    );
    assert!(
        company_domain_allowed(
            &person(Some("a@aifluxx.com"), Some("aifluxx.com"), false),
            "aifluxx.com",
        )
        .is_err()
    );
    assert!(
        company_domain_allowed(
            &person(Some("a@b@aifluxx.com"), Some("aifluxx.com"), true),
            "aifluxx.com",
        )
        .is_err()
    );
}
