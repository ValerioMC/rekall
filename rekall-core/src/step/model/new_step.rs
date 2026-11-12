/// A step to add to a task. `starts_session` makes the step open a session of its own when the flow
/// reaches it.
#[derive(Clone, Copy, Debug, Default)]
pub struct NewStep<'a> {
    pub title: Option<&'a str>,
    pub body_markdown: Option<&'a str>,
    pub starts_session: bool,
}

impl<'a> NewStep<'a> {
    pub fn titled(title: &'a str, body_markdown: Option<&'a str>) -> Self {
        Self { title: Some(title), body_markdown, starts_session: false }
    }
}
