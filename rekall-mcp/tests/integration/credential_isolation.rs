//! Connection credentials must stay out of a session's reach. `rekall-mcp` is built from the whole
//! `Services`, so neither its own code nor `Services` may name the credential store or the types
//! that carry a credential.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: [&str; 5] = ["SecretStore", "KubeAccess", "Kubeconfig", "KubeCredential", "connection::Secret"];

fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(folder).expect("a source folder is readable") {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            found.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
    found
}

fn mentions_of_credentials(files: &[PathBuf]) -> Vec<String> {
    let mut mentions = Vec::new();
    for file in files {
        let text = fs::read_to_string(file).expect("a source file is readable");
        for name in FORBIDDEN.iter().filter(|name| text.contains(*name)) {
            mentions.push(format!("{} names {name}", file.display()));
        }
    }
    mentions
}

#[test]
fn the_mcp_server_never_names_the_credential_store() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mentions = mentions_of_credentials(&rust_files(&source));
    assert!(mentions.is_empty(), "rekall-mcp reaches connection credentials: {mentions:?}");
}

#[test]
fn the_services_the_mcp_server_is_built_from_hold_no_credential_store() {
    let services = Path::new(env!("CARGO_MANIFEST_DIR")).join("../rekall-core/src/services.rs");
    let mentions = mentions_of_credentials(&[services]);
    assert!(mentions.is_empty(), "Services carries connection credentials: {mentions:?}");
}
