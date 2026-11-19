use std::sync::LazyLock;

use regex::Regex;
use rekall_kernel::jstr;

use crate::commit::PendingChange;

static DOCUMENTATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(^|/)(docs?|documentation)/|\.(md|mdx|markdown|rst|txt|adoc)$").expect("a valid pattern"));

static TEST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(^|/)(tests?|__tests__|spec|e2e)/|\.(spec|test)\.[a-z]+$|Tests?\.java$|_test\.(go|py)$|test_[^/]*\.py$")
        .expect("a valid pattern")
});

static FIX_TITLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(fix|bug|hotfix|correggi|corregge|correzione|risolvi|risolve|errore|problema)\b").expect("a valid pattern")
});

static REFACTOR_TITLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(refactor|refactoring|riorganizza|ripulisci)\b").expect("a valid pattern"));

pub(super) fn type_of(title: &str, changes: &[PendingChange]) -> &'static str {
    let plain = jstr::collapse_whitespace(jstr::strip(title));
    if FIX_TITLE.is_match(&plain) {
        return "fix";
    }
    if REFACTOR_TITLE.is_match(&plain) {
        return "refactor";
    }
    if !changes.is_empty() && changes.iter().all(|change| DOCUMENTATION.is_match(&change.path)) {
        return "docs";
    }
    if !changes.is_empty() && changes.iter().all(|change| TEST.is_match(&change.path)) {
        return "test";
    }
    "feat"
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/model/change_classification_tests.rs"]
mod tests;
