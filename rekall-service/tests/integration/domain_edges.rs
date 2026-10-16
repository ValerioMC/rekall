//! Which domain of `rekall-service` may use which. A new edge is a design decision: it is added
//! to `ALLOWED` here and to the table in `docs/DESIGN.md` §3, or the code is changed instead.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// Foundations every domain may use: the transaction, the context, the loaders and the event bus.
const FOUNDATIONS: [&str; 6] = ["ctx", "events", "load", "services", "text_rule", "tx"];

/// Each domain and the domains it may use.
const ALLOWED: [(&str, &[&str]); 16] = [
    ("claude", &[]),
    ("commit", &[]),
    ("component", &[]),
    ("context", &["component", "step", "wrapup"]),
    ("diagram", &[]),
    ("flow", &["step"]),
    ("note", &[]),
    ("review", &["timeentry"]),
    ("revision", &[]),
    ("search", &[]),
    ("section", &["note"]),
    ("step", &["timeentry"]),
    ("task", &["step"]),
    ("timeentry", &[]),
    ("token", &[]),
    ("wrapup", &["review", "revision"]),
];

fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(folder).expect("a domain folder is readable") {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            found.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
    found
}

fn domains(source: &Path) -> BTreeSet<String> {
    ALLOWED.iter().map(|(name, _)| (*name).to_string()).filter(|name| source.join(name).is_dir()).collect()
}

fn used_modules(text: &str) -> BTreeSet<String> {
    let direct = Regex::new(r"crate::(\w+)").expect("the direct pattern is valid");
    let grouped = Regex::new(r"use crate::\{([^}]*)\}").expect("the group pattern is valid");
    let mut used: BTreeSet<String> = direct.captures_iter(text).map(|capture| capture[1].to_string()).collect();
    for group in grouped.captures_iter(text) {
        used.extend(group[1].split(',').map(|item| item.trim().split("::").next().unwrap_or("").to_string()));
    }
    used
}

fn actual_edges(source: &Path) -> BTreeMap<String, BTreeSet<String>> {
    let known = domains(source);
    let mut edges = BTreeMap::new();
    for domain in &known {
        let mut used = BTreeSet::new();
        for file in rust_files(&source.join(domain)) {
            let text = fs::read_to_string(&file).expect("a source file is readable");
            used.extend(used_modules(&text));
        }
        used.retain(|module| known.contains(module) && module != domain && !FOUNDATIONS.contains(&module.as_str()));
        edges.insert(domain.clone(), used);
    }
    edges
}

#[test]
fn every_domain_folder_is_listed() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let on_disk: BTreeSet<String> = fs::read_dir(&source)
        .expect("src is readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !FOUNDATIONS.contains(&name.as_str()))
        .collect();
    let listed: BTreeSet<String> = ALLOWED.iter().map(|(name, _)| (*name).to_string()).collect();
    assert_eq!(on_disk, listed, "a domain folder was added or removed: update ALLOWED");
}

#[test]
fn domains_use_only_the_domains_they_are_allowed_to() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut surprises = Vec::new();
    for (domain, used) in actual_edges(&source) {
        let allowed: BTreeSet<&str> = ALLOWED.iter().find(|(name, _)| *name == domain).map(|(_, uses)| uses.iter().copied().collect()).unwrap_or_default();
        for module in used.iter().filter(|module| !allowed.contains(module.as_str())) {
            surprises.push(format!("{domain} -> {module}"));
        }
    }
    assert!(surprises.is_empty(), "new domain edges, allow them in ALLOWED and DESIGN.md or remove them: {surprises:?}");
}

#[test]
fn no_two_domains_use_each_other() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let edges = actual_edges(&source);
    let mut cycles = Vec::new();
    for (domain, used) in &edges {
        for other in used.iter().filter(|other| edges.get(*other).is_some_and(|back| back.contains(domain)) && *other > domain) {
            cycles.push(format!("{domain} <-> {other}"));
        }
    }
    assert!(cycles.is_empty(), "domains that depend on each other: {cycles:?}");
}
