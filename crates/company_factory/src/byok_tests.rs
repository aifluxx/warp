use super::{ByokError, fingerprint};

#[test]
fn fingerprint_hides_the_key_and_rejects_a_short_value() {
    let key = "alice-personal-model-key";
    let printed = fingerprint(key).unwrap();
    assert_eq!(printed.len(), 12);
    assert!(!printed.contains("alice"));
    assert!(matches!(fingerprint("short-key"), Err(ByokError::TooShort)));
    assert!(matches!(
        fingerprint("  padded-personal-key  "),
        Err(ByokError::Empty)
    ));
}

#[test]
fn the_same_key_always_has_the_same_fingerprint() {
    let key = "alice-personal-model-key";
    assert_eq!(fingerprint(key).unwrap(), fingerprint(key).unwrap());
    assert_ne!(
        fingerprint(key).unwrap(),
        fingerprint("bob-personal-model-key-1").unwrap()
    );
}
