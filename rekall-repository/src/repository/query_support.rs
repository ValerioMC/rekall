use rekall_common::RekallError;

/// Spring Data's `upper(x) = upper(?)`.
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.to_uppercase() == b.to_uppercase()
}

/// `LOWER(x) LIKE LOWER(CONCAT('%', term, '%'))`: whether `text` holds `term`, case folded.
pub fn contains_ignore_case(text: &str, term: &str) -> bool {
    text.to_lowercase().contains(&term.to_lowercase())
}

/// A derived query declared to return `Optional<T>` that found more than one row: Spring Data
/// threw `IncorrectResultSizeDataAccessException`, which nothing handled.
pub fn at_most_one<T>(mut rows: Vec<T>) -> Result<Option<T>, RekallError> {
    match rows.len() {
        0 => Ok(None),
        1 => Ok(rows.pop()),
        n => Err(RekallError::internal(
            "IncorrectResultSizeDataAccessException",
            format!("Query did not return a unique result: {n} results were returned"),
        )),
    }
}
