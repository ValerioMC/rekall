//! What a written inventory holds to. Every refusal names the entry by its position, so a session
//! can correct the list and send it again whole.

use std::collections::HashSet;

use rekall_common::{jstr, Id, Instant, RekallError, Result};
use rekall_model::project_dependency::{self, MANIFEST_MAX_CHARACTERS, NAME_MAX_CHARACTERS, VERSION_MAX_CHARACTERS};
use rekall_model::Ecosystem;

use crate::text_rule;

use super::DeclaredDependency;

/// More than a lockfile of a large monorepo resolves; past it the list is not an inventory.
pub const MAX_DEPENDENCIES: usize = 5_000;

/// The characters that make a version a range a manifest asks for rather than one it resolved to.
const RANGE_MARKS: [char; 9] = ['^', '~', '>', '<', '=', '*', '|', ' ', ','];

/// The rows to store for `project_id`: one per ecosystem, name and version, the first manifest kept.
pub fn dependencies(project_id: Id, declared: &[DeclaredDependency], now: Instant) -> Result<Vec<project_dependency::Model>> {
    if declared.is_empty() {
        return Err(RekallError::illegal("An inventory needs at least one dependency. Read the project's manifests and lockfiles first."));
    }
    if declared.len() > MAX_DEPENDENCIES {
        return Err(RekallError::illegal(format!("An inventory holds at most {MAX_DEPENDENCIES} dependencies and this one has {}.", declared.len())));
    }
    let mut seen = HashSet::new();
    let mut rows = Vec::with_capacity(declared.len());
    for (at, entry) in declared.iter().enumerate() {
        let row = dependency(project_id, entry, now).map_err(|refused| RekallError::illegal(format!("dependencies[{at}]: {}", refused.message())))?;
        if seen.insert((row.ecosystem, row.name.clone(), row.version.clone())) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn dependency(project_id: Id, entry: &DeclaredDependency, now: Instant) -> Result<project_dependency::Model> {
    let row = project_dependency::Model {
        id: Id::random(),
        project_id,
        ecosystem: ecosystem(entry.ecosystem.as_deref())?,
        name: name(entry.name.as_deref())?,
        version: version(entry.version.as_deref())?,
        manifest: manifest(entry.manifest.as_deref())?,
        created_at: now,
    };
    row.validate()?;
    Ok(row)
}

/// The ecosystem by the name the GitHub Advisory Database gives it, ignoring case.
pub fn ecosystem(name: Option<&str>) -> Result<Ecosystem> {
    let given = text_rule::present(name).ok_or_else(|| RekallError::illegal("an ecosystem is required"))?;
    Ecosystem::of_advisory_name(&jstr::lower(given)).ok_or_else(|| {
        RekallError::illegal(format!("'{given}' is not an ecosystem. Use one of: {}", Ecosystem::advisory_names().join(", ")))
    })
}

fn name(name: Option<&str>) -> Result<String> {
    let given = text_rule::present(name).ok_or_else(|| RekallError::illegal("a name is required"))?;
    text_rule::within(given, NAME_MAX_CHARACTERS, |length| format!("a name is capped at {NAME_MAX_CHARACTERS} characters and this one is {length}"))
}

/// The exact version the project resolves to, which is what an advisory's ranges are checked against.
pub fn version(version: Option<&str>) -> Result<String> {
    let given = text_rule::present(version).ok_or_else(|| RekallError::illegal("a version is required"))?;
    if given.contains(RANGE_MARKS) {
        return Err(RekallError::illegal(format!(
            "'{given}' is a range. Write the exact version the lockfile resolves to, or the one the build uses."
        )));
    }
    let bare = given.strip_prefix('v').filter(|rest| rest.starts_with(|c: char| c.is_ascii_digit())).unwrap_or(given);
    text_rule::within(bare, VERSION_MAX_CHARACTERS, |length| format!("a version is capped at {VERSION_MAX_CHARACTERS} characters and this one is {length}"))
}

/// The declaring file, relative to the project's folder and inside it.
fn manifest(path: Option<&str>) -> Result<String> {
    let given = text_rule::present(path).ok_or_else(|| RekallError::illegal("a manifest is required: the file that declares the dependency"))?;
    let normalised = given.replace('\\', "/");
    let relative = normalised.trim_start_matches("./");
    if relative.starts_with('/') || relative.split('/').any(|segment| segment == "..") {
        return Err(RekallError::illegal(format!("'{given}' has to be a path inside the project's folder, relative to it")));
    }
    text_rule::within(relative, MANIFEST_MAX_CHARACTERS, |length| format!("a manifest path is capped at {MANIFEST_MAX_CHARACTERS} characters and this one is {length}"))
}

#[cfg(test)]
#[path = "../../tests/unit/inventory/rules_tests.rs"]
mod tests;
