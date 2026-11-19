//! Confluence's storage format, the XHTML a page body is kept in, written out as markdown. A tag it
//! does not know is dropped and its text kept; a macro's parameters are dropped with their text.

use std::sync::LazyLock;

use regex::Regex;

static TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)<!\[CDATA\[(?P<cdata>.*?)\]\]>|<!--.*?-->|<(?P<closing>/?)(?P<name>[A-Za-z][\w:.-]*)(?P<attributes>[^>]*?)(?P<selfclosing>/?)>|(?P<text>[^<]+)")
        .expect("a valid pattern")
});
static HREF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"href\s*=\s*"([^"]*)""#).expect("a valid pattern"));
static MACRO_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"ac:name\s*=\s*"([^"]*)""#).expect("a valid pattern"));
static WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").expect("a valid pattern"));
static BLANK_LINES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").expect("a valid pattern"));

pub(super) fn to_markdown(storage: &str) -> String {
    let mut writer = MarkdownWriter::default();
    for token in TOKEN.captures_iter(storage) {
        if let Some(code) = token.name("cdata") {
            writer.cdata(code.as_str());
        } else if let Some(text) = token.name("text") {
            writer.text(text.as_str());
        } else if let Some(name) = token.name("name") {
            let name = name.as_str().to_ascii_lowercase();
            let attributes = token.name("attributes").map_or("", |found| found.as_str());
            let closing = token.name("closing").is_some_and(|found| !found.as_str().is_empty());
            let self_closing = token.name("selfclosing").is_some_and(|found| !found.as_str().is_empty());
            if closing {
                writer.close(&name);
            } else {
                writer.open(&name, attributes);
                if self_closing {
                    writer.close(&name);
                }
            }
        }
    }
    BLANK_LINES.replace_all(writer.out.trim(), "\n\n").into_owned()
}

#[derive(Default)]
struct MarkdownWriter {
    out: String,
    lists: Vec<ListState>,
    links: Vec<String>,
    macros: Vec<bool>,
    skipping: usize,
    code_depth: usize,
    table: TableState,
}

struct ListState {
    ordered: bool,
    items: usize,
}

#[derive(Default)]
struct TableState {
    cells_in_row: usize,
    header_written: bool,
}

impl MarkdownWriter {
    fn text(&mut self, raw: &str) {
        if self.skipping > 0 {
            return;
        }
        let decoded = decoded_entities(raw);
        if self.code_depth > 0 {
            self.out.push_str(&decoded);
            return;
        }
        let collapsed = WHITESPACE.replace_all(&decoded, " ");
        let text = if self.out.is_empty() || self.out.ends_with(['\n', ' ']) { collapsed.trim_start().to_string() } else { collapsed.into_owned() };
        self.out.push_str(&text);
    }

    fn cdata(&mut self, raw: &str) {
        if self.skipping == 0 {
            self.out.push_str(raw);
        }
    }

    fn open(&mut self, name: &str, attributes: &str) {
        match name {
            "ac:parameter" => self.skipping += 1,
            "ac:structured-macro" => self.open_macro(attributes),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.block_break();
                self.out.push_str(&format!("{} ", "#".repeat(usize::from(name.as_bytes()[1] - b'0'))));
            }
            "p" | "div" | "blockquote" | "pre" => self.block_break(),
            "br" => self.out.push('\n'),
            "hr" => {
                self.block_break();
                self.out.push_str("---");
                self.block_break();
            }
            "ul" | "ol" => self.lists.push(ListState { ordered: name == "ol", items: 0 }),
            "li" => self.open_item(),
            "strong" | "b" => self.out.push_str("**"),
            "em" | "i" => self.out.push('*'),
            "code" if self.code_depth == 0 => self.out.push('`'),
            "a" => self.open_link(attributes),
            "table" => {
                self.block_break();
                self.table = TableState::default();
            }
            "tr" => self.table.cells_in_row = 0,
            "th" | "td" => {
                self.table.cells_in_row += 1;
                self.out.push_str("| ");
            }
            _ => {}
        }
    }

    fn close(&mut self, name: &str) {
        match name {
            "ac:parameter" => self.skipping = self.skipping.saturating_sub(1),
            "ac:structured-macro" => self.close_macro(),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "div" | "blockquote" | "pre" => self.block_break(),
            "ul" | "ol" => {
                self.lists.pop();
                if self.lists.is_empty() {
                    self.block_break();
                }
            }
            "strong" | "b" => self.out.push_str("**"),
            "em" | "i" => self.out.push('*'),
            "code" if self.code_depth == 0 => self.out.push('`'),
            "a" => self.close_link(),
            "th" | "td" => self.out.push(' '),
            "tr" => self.close_row(),
            "table" => self.block_break(),
            _ => {}
        }
    }

    fn open_macro(&mut self, attributes: &str) {
        let is_code = MACRO_NAME.captures(attributes).is_some_and(|name| matches!(&name[1], "code" | "noformat"));
        self.macros.push(is_code);
        if is_code {
            self.block_break();
            self.out.push_str("```\n");
            self.code_depth += 1;
        }
    }

    fn close_macro(&mut self) {
        if self.macros.pop().unwrap_or(false) {
            self.code_depth = self.code_depth.saturating_sub(1);
            if !self.out.ends_with('\n') {
                self.out.push('\n');
            }
            self.out.push_str("```");
            self.block_break();
        }
    }

    fn open_item(&mut self) {
        let depth = self.lists.len().saturating_sub(1);
        let Some(list) = self.lists.last_mut() else { return };
        list.items += 1;
        let marker = if list.ordered { format!("{}. ", list.items) } else { "- ".to_string() };
        if !self.out.ends_with('\n') {
            self.out.push('\n');
        }
        self.out.push_str(&format!("{}{marker}", "  ".repeat(depth)));
    }

    fn open_link(&mut self, attributes: &str) {
        let href = HREF.captures(attributes).map(|found| decoded_entities(&found[1])).unwrap_or_default();
        if !href.is_empty() {
            self.out.push('[');
        }
        self.links.push(href);
    }

    fn close_link(&mut self) {
        if let Some(href) = self.links.pop().filter(|href| !href.is_empty()) {
            self.out.push_str(&format!("]({href})"));
        }
    }

    fn close_row(&mut self) {
        self.out.push_str("|\n");
        if !self.table.header_written {
            self.table.header_written = true;
            self.out.push_str(&format!("{}|\n", "|---".repeat(self.table.cells_in_row)));
        }
    }

    fn block_break(&mut self) {
        let trimmed = self.out.trim_end_matches(' ').len();
        self.out.truncate(trimmed);
        if self.out.is_empty() || self.out.ends_with("\n\n") {
            return;
        }
        self.out.push_str(if self.out.ends_with('\n') { "\n" } else { "\n\n" });
    }
}

fn decoded_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&rsquo;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
#[path = "../../tests/unit/confluence/storage_format_tests.rs"]
mod tests;
