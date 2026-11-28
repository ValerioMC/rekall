//! How a refusal names the files that stand in its way.

const FILES_NAMED: usize = 5;

/// The count and the first few files, so a long list stays readable.
pub fn named_uncommitted_files(files: &[String]) -> String {
    let count = files.len();
    let plural = if count == 1 { "" } else { "s" };
    let mut named = files.iter().take(FILES_NAMED).cloned().collect::<Vec<_>>().join(", ");
    if count > FILES_NAMED {
        named.push_str(&format!(" and {} more", count - FILES_NAMED));
    }
    format!("{count} uncommitted file{plural}: {named}")
}

#[cfg(test)]
#[path = "../tests/unit/file_list_tests.rs"]
mod tests;
