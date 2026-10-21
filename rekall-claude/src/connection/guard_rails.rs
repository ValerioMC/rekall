//! The Claude Code settings a credential session starts with: deny rules that stop a session from
//! reading its helpers' folder, dumping its environment or calling a helper by hand. Pattern
//! matches against an accidental read, not a sandbox; they hold under `--dangerously-skip-permissions`.

use std::path::Path;

use serde::Serialize;

use super::helper_scripts::{GIT_HELPER, KUBE_PLUGIN};

#[derive(Serialize)]
struct Settings {
    permissions: Permissions,
}

#[derive(Serialize)]
struct Permissions {
    deny: Vec<String>,
}

/// `folder` is the session's own, `endpoint` where its helpers ask.
pub(super) fn deny_rules(folder: &Path, endpoint: &str) -> Vec<String> {
    let folder = folder.display().to_string();
    let mut rules = vec![
        // A leading `//` makes a file rule absolute.
        format!("Read(/{folder}/**)"),
        format!("Edit(/{folder}/**)"),
        "Bash(env)".into(),
        "Bash(printenv:*)".into(),
        "Bash(export -p)".into(),
        "Bash(set)".into(),
        "Bash(git credential:*)".into(),
        "Bash(git credential-cache:*)".into(),
        "Bash(kubectl config view --raw:*)".into(),
        "Bash(security find-generic-password:*)".into(),
        "Bash(security dump-keychain:*)".into(),
    ];
    for helper in [KUBE_PLUGIN, GIT_HELPER] {
        rules.push(format!("Bash({helper}:*)"));
        rules.push(format!("Bash({folder}/{helper}:*)"));
    }
    for reader in ["cat", "head", "tail", "less", "more"] {
        rules.push(format!("Bash({reader} {folder}:*)"));
    }
    for client in ["curl", "wget"] {
        rules.push(format!("Bash({client} {endpoint}:*)"));
    }
    rules
}

pub(super) fn settings_document(folder: &Path, endpoint: &str) -> String {
    let settings = Settings { permissions: Permissions { deny: deny_rules(folder, endpoint) } };
    serde_json::to_string_pretty(&settings).expect("a list of strings always serialises")
}

#[cfg(test)]
#[path = "../../tests/unit/connection/guard_rails_tests.rs"]
mod tests;
