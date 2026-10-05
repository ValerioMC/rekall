use rekall_common::{InternalFailure, RekallError};

/// Equal once both sides are upper-cased, across all of Unicode.
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.to_uppercase() == b.to_uppercase()
}

/// `LOWER(x) LIKE LOWER(CONCAT('%', term, '%'))`: whether `text` holds `term`, case folded.
pub fn contains_ignore_case(text: &str, term: &str) -> bool {
    text.to_lowercase().contains(&term.to_lowercase())
}

/// A lookup that expects one row at most. More than one means a uniqueness rule the schema does
/// not enforce was broken, which no caller can recover from.
pub fn at_most_one<T>(mut rows: Vec<T>) -> Result<Option<T>, RekallError> {
    match rows.len() {
        0 => Ok(None),
        1 => Ok(rows.pop()),
        n => Err(RekallError::internal(InternalFailure::Integrity,
            format!("Query did not return a unique result: {n} results were returned"),
        )),
    }
}
