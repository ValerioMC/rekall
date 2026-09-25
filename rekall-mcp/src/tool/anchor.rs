//! The anchor syntax: the parsing surface of `/rk`, where a silent misparse loads the wrong record.

use std::sync::LazyLock;

use regex::Regex;

use crate::protocol::ToolError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub entity_name: Option<String>,
    pub value: String,
}

impl Anchor {
    pub fn new(entity_name: Option<&str>, value: &str) -> Self {
        Self { entity_name: entity_name.map(str::to_string), value: value.to_string() }
    }

    /// Every term in `raw`: whitespace-separated, a quoted value kept whole.
    pub fn parse_all(raw: Option<&str>) -> Result<Vec<Anchor>, ToolError> {
        // `\S*"[^"]*"|\S+`, with Java's ASCII `\S`.
        static TERM: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"[^ \t\n\x0B\x0C\r]*"[^"]*"|[^ \t\n\x0B\x0C\r]+"#).unwrap()
        });
        let mut anchors = Vec::new();
        for term in TERM.find_iter(raw.unwrap_or("")) {
            anchors.push(Self::parse(term.as_str())?);
        }
        if anchors.is_empty() {
            return Err(ToolError::Failure(
                "No anchor given. Pass something like `project:vega task:report-builder`.".into(),
            ));
        }
        Ok(anchors)
    }

    fn parse(term: &str) -> Result<Anchor, ToolError> {
        let separator = term.find(':');
        let quote = term.find('"');
        let qualified = match (separator, quote) {
            (Some(s), None) => Some(s),
            (Some(s), Some(q)) if s < q => Some(s),
            _ => None,
        };
        let Some(separator) = qualified else {
            return Ok(Anchor { entity_name: None, value: unquote(term) });
        };
        let entity_name = rekall_common::jstr::trim(&term[..separator]);
        let value = unquote(rekall_common::jstr::trim(&term[separator + 1..]));
        if entity_name.is_empty() || value.is_empty() {
            return Err(ToolError::Failure(format!(
                "'{term}' is not a valid anchor. Use `entity:value`, or a bare value."
            )));
        }
        Ok(Anchor { entity_name: Some(entity_name.to_string()), value })
    }

    pub fn is_qualified(&self) -> bool {
        self.entity_name.is_some()
    }

    pub fn is(&self, entity: &str) -> bool {
        self.entity_name.as_deref().is_some_and(|name| rekall_common::jstr::equals_ignore_case(name, entity))
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.entity_name {
            Some(entity) => write!(f, "{entity}:{}", self.value),
            None => f.write_str(&self.value),
        }
    }
}

fn unquote(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return rekall_common::jstr::trim(&value[1..value.len() - 1]).to_string();
    }
    value.to_string()
}

/// A write's target: exactly one task, with the project that disambiguates it when given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchoredTask {
    pub project_label: Option<String>,
    pub task_label: String,
}

impl AnchoredTask {
    pub fn from(anchors: &[Anchor]) -> Result<Self, ToolError> {
        let mut project = None;
        let mut task: Option<String> = None;
        let mut bare = Vec::new();
        for anchor in anchors {
            if anchor.is("task") {
                if task.is_some() {
                    return Err(ToolError::Failure("Two task anchors were given. This write belongs to one task.".into()));
                }
                task = Some(anchor.value.clone());
            } else if anchor.is("project") {
                project = Some(anchor.value.clone());
            } else if !anchor.is_qualified() {
                bare.push(anchor.value.clone());
            } else {
                return Err(ToolError::Failure(format!(
                    "`{anchor}` cannot say which task to write to. Pass `project:<label> task:<label>`."
                )));
            }
        }
        if task.is_none() && bare.len() == 1 {
            task = bare.pop();
        }
        let Some(task) = task else {
            return Err(ToolError::Failure(
                "No task in those anchors. This write belongs to exactly one task: pass `project:<label> task:<label>`."
                    .into(),
            ));
        };
        Ok(Self { project_label: project, task_label: task })
    }

    pub fn anchor(&self) -> String {
        match &self.project_label {
            None => format!("task:{}", self.task_label),
            Some(project) => format!("project:{project} task:{}", self.task_label),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Vec<Anchor> {
        Anchor::parse_all(Some(raw)).unwrap()
    }

    fn failure(raw: &str) -> String {
        match Anchor::parse_all(Some(raw)).unwrap_err() {
            ToolError::Failure(message) => message,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_canonical_form_splits_into_entity_and_value() {
        assert_eq!(parse("project:vega task:report-builder"), [Anchor::new(Some("project"), "vega"), Anchor::new(Some("task"), "report-builder")]);
    }

    #[test]
    fn a_term_without_a_qualifier_is_positional() {
        assert_eq!(parse("vega report-builder"), [Anchor::new(None, "vega"), Anchor::new(None, "report-builder")]);
    }

    #[test]
    fn the_two_forms_mix_in_one_request() {
        assert_eq!(parse("vega task:report-builder"), [Anchor::new(None, "vega"), Anchor::new(Some("task"), "report-builder")]);
    }

    #[test]
    fn a_quoted_value_keeps_its_spaces() {
        assert_eq!(parse("task:\"report builder\" project:vega"), [Anchor::new(Some("task"), "report builder"), Anchor::new(Some("project"), "vega")]);
    }

    #[test]
    fn a_colon_inside_a_quoted_value_belongs_to_the_value() {
        assert_eq!(parse("\"ESA-4412: main workflow\""), [Anchor::new(None, "ESA-4412: main workflow")]);
    }

    #[test]
    fn only_the_first_colon_qualifies_so_a_url_survives_as_a_value() {
        assert_eq!(parse("repo:https://gitlab.example/vega"), [Anchor::new(Some("repo"), "https://gitlab.example/vega")]);
    }

    #[test]
    fn irregular_spacing_is_not_a_syntax_error() {
        assert_eq!(parse("  project:vega   task:report-builder \n"), [Anchor::new(Some("project"), "vega"), Anchor::new(Some("task"), "report-builder")]);
    }

    #[test]
    fn an_empty_request_is_refused_rather_than_answered_with_everything() {
        assert!(failure("   ").contains("project:vega"));
    }

    #[test]
    fn a_half_written_anchor_is_refused_rather_than_read_as_positional() {
        assert!(failure("project:").contains("not a valid anchor"));
        assert!(failure(":vega").contains("not a valid anchor"));
    }
}
