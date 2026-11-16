use std::fs;

use super::*;

fn folder_with(files: &[(&str, &str)]) -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    for (name, content) in files {
        fs::write(folder.path().join(name), content).unwrap();
    }
    folder
}

#[test]
fn a_manifests_description_is_what_the_repository_does() {
    let cargo = folder_with(&[("Cargo.toml", "[package]\nname = \"billing\"\ndescription = \"Issues invoices.\"\n")]);
    let node = folder_with(&[("package.json", r#"{ "name": "console", "description": "The web console." }"#)]);
    let python = folder_with(&[("pyproject.toml", "[project]\nname = \"etl\"\ndescription = \"Loads the ledger.\"\n")]);

    assert_eq!(describe(cargo.path()), "Issues invoices.");
    assert_eq!(describe(node.path()), "The web console.");
    assert_eq!(describe(python.path()), "Loads the ledger.");
}

#[test]
fn a_manifest_beats_the_readme() {
    let folder = folder_with(&[("Cargo.toml", "[package]\ndescription = \"From the manifest.\"\n"), ("README.md", "From the readme.\n")]);

    assert_eq!(describe(folder.path()), "From the manifest.");
}

#[test]
fn the_first_paragraph_of_prose_in_the_readme_is_the_fallback() {
    let readme = "# Billing\n\n[![CI](https://ci/badge.svg)](https://ci)\n\n![logo](logo.png)\n\nIssues invoices\nand sends them out.\n\n## Usage\n\nRun it.\n";
    let folder = folder_with(&[("README.md", readme)]);

    assert_eq!(describe(folder.path()), "Issues invoices and sends them out.");
}

#[test]
fn an_underlined_heading_and_fenced_code_are_not_prose() {
    let readme = "Billing\n=======\n\n```\ncargo run\n```\n\nIssues invoices.\n";
    let folder = folder_with(&[("README.md", readme)]);

    assert_eq!(describe(folder.path()), "Issues invoices.");
}

#[test]
fn a_description_inherited_from_a_workspace_falls_back_to_the_readme() {
    let folder = folder_with(&[("Cargo.toml", "[package]\ndescription = { workspace = true }\n"), ("README.md", "Issues invoices.\n")]);

    assert_eq!(describe(folder.path()), "Issues invoices.");
}

#[test]
fn a_manifest_that_does_not_parse_is_passed_over() {
    let folder = folder_with(&[("package.json", "{ not json"), ("README.md", "Issues invoices.\n")]);

    assert_eq!(describe(folder.path()), "Issues invoices.");
}

#[test]
fn a_repository_that_says_nothing_has_no_description() {
    assert_eq!(describe(folder_with(&[]).path()), "");
    assert_eq!(describe(folder_with(&[("README.md", "# Billing\n")]).path()), "");
}

#[test]
fn a_long_description_is_cut_at_a_word() {
    let sentence = "word ".repeat(100);
    let folder = folder_with(&[("README.md", &sentence)]);

    let described = describe(folder.path());

    assert!(described.ends_with("word…"), "{described}");
    assert!(described.chars().count() <= MAX_CHARACTERS + 1);
}
