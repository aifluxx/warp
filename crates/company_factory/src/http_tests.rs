use super::escape;

#[test]
fn names_from_the_identity_service_cannot_break_the_page() {
    assert_eq!(escape("a<b>@example.com"), "a&lt;b&gt;@example.com");
    assert!(!escape("a<b>@example.com").contains("<b>"));
}
