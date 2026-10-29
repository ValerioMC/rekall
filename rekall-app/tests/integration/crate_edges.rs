//! Which crate of the workspace may list which Rekall crate in its `Cargo.toml`, in any dependency
//! section, dev and target-specific ones included. A new edge is a design decision: it is added
//! here and to `docs/DESIGN.md` §3, or the manifest is changed instead.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// Each workspace member, by its folder, and the Rekall crates it may depend on.
const ALLOWED: [(&str, &[&str]); 13] = [
    ("rekall-common", &[]),
    ("rekall-web", &["rekall-common"]),
    ("rekall-model", &["rekall-common"]),
    ("rekall-diagram", &[]),
    ("rekall-repository", &["rekall-common", "rekall-model"]),
    ("rekall-service", &["rekall-common", "rekall-diagram", "rekall-model", "rekall-repository"]),
    ("rekall-console", &["rekall-common", "rekall-model", "rekall-repository", "rekall-service"]),
    ("rekall-api", &["rekall-common", "rekall-console", "rekall-model", "rekall-repository", "rekall-service", "rekall-web"]),
    ("rekall-mcp", &["rekall-common", "rekall-service"]),
    ("rekall-claude", &["rekall-common", "rekall-diagram", "rekall-model", "rekall-repository", "rekall-service", "rekall-web"]),
    ("rekall-platform", &["rekall-common", "rekall-model", "rekall-repository", "rekall-service", "rekall-web"]),
    (
        "rekall-app",
        &["rekall-api", "rekall-claude", "rekall-common", "rekall-mcp", "rekall-model", "rekall-platform", "rekall-repository", "rekall-service", "rekall-web"],
    ),
    ("rekall-app/desktop", &["rekall-app", "rekall-platform"]),
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("rekall-app sits in the workspace").to_path_buf()
}

fn members() -> BTreeSet<String> {
    let manifest = fs::read_to_string(workspace_root().join("Cargo.toml")).expect("the workspace manifest is readable");
    let block = Regex::new(r"(?s)\nmembers\s*=\s*\[(.*?)\]").expect("the members pattern is valid");
    let quoted = Regex::new(r#""([^"]+)""#).expect("the quoted pattern is valid");
    let list = block.captures(&manifest).expect("the workspace lists its members");
    quoted.captures_iter(&list[1]).map(|capture| capture[1].to_string()).collect()
}

fn rekall_dependencies(member: &str) -> BTreeSet<String> {
    let manifest = fs::read_to_string(workspace_root().join(member).join("Cargo.toml")).expect("a member manifest is readable");
    let mut in_dependencies = false;
    let mut found = BTreeSet::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_dependencies = line.ends_with("dependencies]");
        } else if in_dependencies && line.starts_with("rekall-") {
            found.insert(line.split(['.', ' ', '=']).next().unwrap_or(line).to_string());
        }
    }
    found
}

#[test]
fn every_workspace_member_is_listed() {
    let listed: BTreeSet<String> = ALLOWED.iter().map(|(member, _)| (*member).to_string()).collect();
    assert_eq!(members(), listed, "a crate was added to or removed from the workspace: update ALLOWED");
}

#[test]
fn crates_depend_only_on_the_crates_they_are_allowed_to() {
    let mut surprises = Vec::new();
    for (member, allowed) in ALLOWED {
        let allowed: BTreeSet<&str> = allowed.iter().copied().collect();
        surprises.extend(rekall_dependencies(member).into_iter().filter(|dependency| !allowed.contains(dependency.as_str())).map(|dependency| format!("{member} -> {dependency}")));
    }
    assert!(surprises.is_empty(), "new crate edges, allow them in ALLOWED and DESIGN.md or remove them: {surprises:#?}");
}
