//! Atlassian's document format, the JSON tree Jira's API returns an issue description in, written
//! out as markdown. A node it does not know is written as its content.

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(super) struct AdfNode {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    content: Vec<AdfNode>,
    #[serde(default)]
    attrs: Option<AdfAttrs>,
    #[serde(default)]
    marks: Vec<AdfMark>,
}

/// Only the attributes the markdown needs; the others are ignored.
#[derive(Debug, Default, Deserialize)]
struct AdfAttrs {
    level: Option<u8>,
    language: Option<String>,
    text: Option<String>,
    url: Option<String>,
    href: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdfMark {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    attrs: Option<AdfAttrs>,
}

impl AdfNode {
    pub(super) fn to_markdown(&self) -> String {
        self.block().trim().to_string()
    }

    fn block(&self) -> String {
        match self.kind.as_str() {
            "paragraph" => self.inline(),
            "heading" => format!("{} {}", "#".repeat(usize::from(self.attr_level().clamp(1, 6))), self.inline()),
            "bulletList" => self.list_items(|_| "- ".to_string()),
            "orderedList" => self.list_items(|position| format!("{}. ", position + 1)),
            "codeBlock" => format!("```{}\n{}\n```", self.attrs.as_ref().and_then(|attrs| attrs.language.as_deref()).unwrap_or_default(), self.inline()),
            "blockquote" => self.blocks().lines().map(|line| format!("> {line}")).collect::<Vec<_>>().join("\n"),
            "rule" => "---".to_string(),
            "text" | "hardBreak" | "mention" | "emoji" | "inlineCard" => self.inline(),
            _ => self.blocks(),
        }
    }

    fn blocks(&self) -> String {
        self.content.iter().map(AdfNode::block).filter(|block| !block.is_empty()).collect::<Vec<_>>().join("\n\n")
    }

    /// Each item's first line after its marker, the lines after it indented under it.
    fn list_items(&self, marker: impl Fn(usize) -> String) -> String {
        let mut lines = Vec::new();
        for (position, item) in self.content.iter().enumerate() {
            let text = item.content.iter().map(AdfNode::block).collect::<Vec<_>>().join("\n");
            let mut item_lines = text.lines();
            lines.push(format!("{}{}", marker(position), item_lines.next().unwrap_or_default()));
            lines.extend(item_lines.map(|line| format!("  {line}")));
        }
        lines.join("\n")
    }

    fn inline(&self) -> String {
        match self.kind.as_str() {
            "text" => self.marked_text(),
            "hardBreak" => "\n".to_string(),
            "mention" | "emoji" => self.attrs.as_ref().and_then(|attrs| attrs.text.clone()).unwrap_or_default(),
            "inlineCard" => self.attrs.as_ref().and_then(|attrs| attrs.url.clone()).unwrap_or_default(),
            _ => self.content.iter().map(AdfNode::inline).collect(),
        }
    }

    fn marked_text(&self) -> String {
        let mut text = self.text.clone().unwrap_or_default();
        for mark in &self.marks {
            text = match mark.kind.as_str() {
                "code" => format!("`{text}`"),
                "strong" => format!("**{text}**"),
                "em" => format!("_{text}_"),
                "strike" => format!("~~{text}~~"),
                "link" => match mark.attrs.as_ref().and_then(|attrs| attrs.href.as_deref()) {
                    Some(href) => format!("[{text}]({href})"),
                    None => text,
                },
                _ => text,
            };
        }
        text
    }

    fn attr_level(&self) -> u8 {
        self.attrs.as_ref().and_then(|attrs| attrs.level).unwrap_or(1)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/jira/atlassian_document_tests.rs"]
mod tests;
