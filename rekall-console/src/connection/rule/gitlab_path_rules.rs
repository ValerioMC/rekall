//! What a GitLab path typed for a binding is read as: a plain `group/project` path whatever was
//! pasted, whether it sits under the connection's root path, and the sibling a misspelling is nearest to.

use rekall_core::connection::GitlabNamespace;

const NEAREST_MAX_DISTANCE: usize = 3;

/// The path inside a pasted address: the scheme, host, `.git` suffix and outer slashes are dropped.
pub(in crate::connection) fn path_of(raw: &str) -> String {
    let typed = raw.trim();
    let without_scheme = typed.strip_prefix("https://").or_else(|| typed.strip_prefix("http://"));
    let path = match without_scheme {
        Some(address) => address.split_once('/').map_or("", |(_, path)| path),
        None => typed,
    };
    path.trim_matches('/').trim_end_matches(".git").to_string()
}

/// Whether `path` is the root path or below it. GitLab paths are case-insensitive; no root path covers the instance.
pub(in crate::connection) fn is_within_root(root: Option<&str>, path: &str) -> bool {
    let Some(root) = root.map(|root| root.trim_matches('/')).filter(|root| !root.is_empty()) else { return true };
    let (root, path) = (root.to_lowercase(), path.to_lowercase());
    path == root || path.starts_with(&format!("{root}/"))
}

/// The group a path sits in, none for a top-level one.
pub(in crate::connection) fn parent_of(path: &str) -> Option<&str> {
    path.rsplit_once('/').map(|(parent, _)| parent)
}

/// The sibling whose name is closest to the last segment typed, when it is close enough to be a misspelling of it.
pub(in crate::connection) fn nearest_sibling(path: &str, siblings: &[GitlabNamespace]) -> Option<String> {
    let typed = path.rsplit('/').next().unwrap_or(path).to_lowercase();
    siblings
        .iter()
        .map(|sibling| (distance(&typed, sibling.path.rsplit('/').next().unwrap_or(&sibling.path).to_lowercase().as_str()), sibling))
        .filter(|(distance, _)| *distance <= NEAREST_MAX_DISTANCE)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, sibling)| sibling.path.clone())
}

fn distance(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (row, left_char) in left.chars().enumerate() {
        let mut current = vec![row + 1];
        for (column, right_char) in right.iter().enumerate() {
            let substitution = previous[column] + usize::from(left_char != *right_char);
            current.push(substitution.min(previous[column + 1] + 1).min(current[column] + 1));
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/rule/gitlab_path_rules_tests.rs"]
mod tests;
