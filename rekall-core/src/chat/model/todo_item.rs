use serde::Serialize;

use crate::chat::TodoStatus;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
}
