use std::fs;

use tempfile::tempdir;

use super::{RepoError, require_assigned, visible_names};

#[test]
fn a_different_repository_is_not_assigned() {
    assert_eq!(require_assigned("warp", "warp"), Ok("warp"));
    assert_eq!(
        require_assigned("../other", "warp"),
        Err(RepoError::NotAssigned)
    );
    assert_eq!(
        require_assigned("warp-other", "warp"),
        Err(RepoError::NotAssigned)
    );
}

#[test]
fn inventory_lists_visible_names_only() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("notes.txt"), "hello").unwrap();
    fs::write(root.path().join(".secret"), "hidden").unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    let names = visible_names(root.path()).unwrap();
    assert_eq!(names, vec!["notes.txt".to_string(), "src".to_string()]);
}
