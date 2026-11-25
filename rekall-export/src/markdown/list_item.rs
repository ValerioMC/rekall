use super::Block;

/// `checked` is set only for a task-list item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

impl ListItem {
    /// The text a writer prints before a task-list item, which neither body font has a glyph for.
    pub fn task_marker(&self) -> Option<&'static str> {
        self.checked.map(|checked| if checked { "[x] " } else { "[ ] " })
    }
}
