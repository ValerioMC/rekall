/// The `name`, `description` and `type` of a markdown file's leading `---` block. A line reader,
/// not a YAML parser: `type` is found at the top level or one level down, under `metadata`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>,
}

const FENCE: &str = "---";
const FOLDED: [&str; 4] = [">", "|", ">-", "|-"];

impl Frontmatter {
    pub fn parse(text: &str) -> Self {
        let mut lines = text.trim_start_matches('\u{feff}').lines();
        if lines.next().map(str::trim_end) != Some(FENCE) {
            return Self::default();
        }
        let block: Vec<&str> = lines.take_while(|line| line.trim_end() != FENCE).collect();
        let mut found = Self::default();
        for (at, line) in block.iter().enumerate() {
            let Some((key, value)) = line.trim_start().split_once(':') else { continue };
            let slot = match key {
                "name" => &mut found.name,
                "description" => &mut found.description,
                "type" => &mut found.kind,
                _ => continue,
            };
            if slot.is_none() {
                *slot = value_at(value.trim(), &block[at + 1..]);
            }
        }
        found
    }
}

/// A folded or literal scalar takes the indented lines under it.
fn value_at(value: &str, following: &[&str]) -> Option<String> {
    let text = if FOLDED.contains(&value) {
        following
            .iter()
            .take_while(|line| line.starts_with(' ') || line.starts_with('\t'))
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        unquote(value).to_string()
    };
    (!text.is_empty()).then_some(text)
}

fn unquote(value: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = value.strip_prefix(quote).and_then(|rest| rest.strip_suffix(quote)) {
            return inner;
        }
    }
    value
}

#[cfg(test)]
#[path = "../../tests/unit/resources/frontmatter_tests.rs"]
mod tests;
