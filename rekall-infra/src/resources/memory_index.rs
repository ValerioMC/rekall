//! Edits to a `MEMORY.md` index, where each memory is one `- [Title](file.md) — hook` line.
//! A memory deleted without its line leaves a dangling pointer Claude would try to follow.

/// The index with a line for `file_name`, unless one already links it.
pub fn with_entry(index: &str, file_name: &str, title: &str, hook: &str) -> String {
    if index.lines().any(|line| links_to(line, file_name)) {
        return index.to_string();
    }
    let line = if hook.is_empty() { format!("- [{title}]({file_name})") } else { format!("- [{title}]({file_name}) — {hook}") };
    let kept = index.trim_end_matches('\n');
    if kept.is_empty() { format!("{line}\n") } else { format!("{kept}\n{line}\n") }
}

/// The index without the lines that link `file_name`; the same text when none does.
pub fn without_entry(index: &str, file_name: &str) -> String {
    if !index.lines().any(|line| links_to(line, file_name)) {
        return index.to_string();
    }
    let kept: Vec<&str> = index.lines().filter(|line| !links_to(line, file_name)).collect();
    if kept.is_empty() { String::new() } else { format!("{}\n", kept.join("\n")) }
}

fn links_to(line: &str, file_name: &str) -> bool {
    line.contains(&format!("]({file_name})"))
}

#[cfg(test)]
#[path = "../../tests/unit/resources/memory_index_tests.rs"]
mod tests;
