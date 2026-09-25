//! A label: what an anchor resolves. Normalised on write rather than refused, because what someone
//! types into a label field is a name and what the anchor needs is an identifier.

use std::sync::LazyLock;

use regex::Regex;
use rekall_common::{jstr, RekallError};

pub struct Slug;

impl Slug {
    pub const PATTERN: &'static str = r"^[a-z0-9]+([._-][a-z0-9]+)*$";

    /// `Slug.of`: trimmed, lower-cased, every run of anything else turned into one `-`, runs of
    /// separators folded, and separators at either end dropped. Refuses a value with nothing
    /// usable left in it.
    pub fn of(raw: Option<&str>) -> Result<String, RekallError> {
        static OTHER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9._-]+").unwrap());
        static RUNS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[._-]{2,}").unwrap());
        static EDGES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[._-]+|[._-]+$").unwrap());

        let Some(raw) = raw else {
            return Err(RekallError::illegal("A label is required. It is what `/rk` looks up."));
        };
        let lowered = jstr::lower(jstr::trim(raw));
        let replaced = OTHER.replace_all(&lowered, "-");
        let folded = RUNS.replace_all(&replaced, "-");
        let slug = EDGES.replace_all(&folded, "").into_owned();
        if slug.is_empty() {
            return Err(RekallError::illegal(format!(
                "'{raw}' leaves no usable label. Use letters, digits, '-', '_' or '.'"
            )));
        }
        Ok(slug)
    }

    pub fn matches(value: &str) -> bool {
        static PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(Slug::PATTERN).unwrap());
        PATTERN.is_match(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_person_types_becomes_a_term_an_anchor_can_carry() {
        for (raw, expected) in [
            ("vega", "vega"),
            ("Vega", "vega"),
            ("  Vega Platform  ", "vega-platform"),
            ("Report Builder", "report-builder"),
            ("report-builder", "report-builder"),
            ("release_1.2", "release_1.2"),
            ("a  //  b", "a-b"),
            ("--edge--", "edge"),
            // An accent is dropped, not transliterated: a label is an identifier.
            ("caffè", "caff"),
        ] {
            assert_eq!(Slug::of(Some(raw)).unwrap(), expected, "{raw}");
        }
    }

    #[test]
    fn the_result_is_always_a_valid_label_whatever_went_in() {
        for raw in ["Vega", "a / b", "..hidden..", "x--y", "1"] {
            assert!(Slug::matches(&Slug::of(Some(raw)).unwrap()), "{raw}");
        }
    }

    #[test]
    fn a_value_with_nothing_usable_in_it_is_refused_rather_than_silently_emptied() {
        for raw in ["", "   ", "///", "...", "---"] {
            assert!(Slug::of(Some(raw)).unwrap_err().is_illegal_argument(), "{raw}");
        }
    }

    #[test]
    fn refuses_null() {
        assert!(Slug::of(None).unwrap_err().is_illegal_argument());
    }
}
