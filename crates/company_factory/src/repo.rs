use std::{fs, path::Path};

#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    NotAssigned,
    Unreadable,
}

pub fn require_assigned<'a>(requested: &'a str, assigned: &'a str) -> Result<&'a str, RepoError> {
    if requested == assigned {
        Ok(requested)
    } else {
        Err(RepoError::NotAssigned)
    }
}

/// Visible entry names directly inside the assigned repository.
/// Hidden names are omitted and file contents are not read.
pub fn visible_names(root: &Path) -> Result<Vec<String>, RepoError> {
    let entries = fs::read_dir(root).map_err(|_| RepoError::Unreadable)?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| RepoError::Unreadable)?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        names.push(name.to_string());
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
#[path = "repo_tests.rs"]
mod tests;
