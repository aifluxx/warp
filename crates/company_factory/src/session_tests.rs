use super::SessionStore;

fn store() -> SessionStore {
    SessionStore::new(b"test-session-secret-is-32-bytes!".to_vec(), false)
}

#[test]
fn a_tampered_cookie_does_not_restore_a_session() {
    let store = store();
    let header = store.issue("person".to_string(), None, "warp".to_string());
    let mut value = header.split(';').next().unwrap().to_string();
    let last = value.pop().unwrap();
    value.push(if last == 'a' { 'b' } else { 'a' });
    assert!(store.read(Some(&value)).is_none());
}

#[test]
fn logout_drops_the_personal_key() {
    let store = store();
    let header = store.issue(
        "person".to_string(),
        Some("a@example.com".to_string()),
        "warp".to_string(),
    );
    let view = store.read(Some(&header)).unwrap();
    store
        .set_byok(&view.id, "alice-personal-model-key")
        .unwrap();
    assert!(store.has_byok(&view.id));
    let cleared = store.logout(Some(&header));
    assert!(cleared.contains("Max-Age=0"));
    assert!(store.read(Some(&header)).is_none());
    assert!(!store.has_byok(&view.id));
}
