use async_trait::async_trait;
use rekall_common::RekallError;
use rekall_service::context::{ContextRecord, ContextRenderer};
use rekall_service::Services;
use serde_json::Value;

use super::{Anchor, Arguments};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

pub struct ContextTool {
    services: Services,
}

impl ContextTool {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    async fn load(&self, anchors: &[Anchor]) -> Result<Vec<ContextRecord>, RekallError> {
        let context = &self.services.context;
        if anchors.len() == 2 && anchors[0].is("project") && anchors[1].is("task") {
            return Ok(vec![
                context.load(Some("project"), &anchors[0].value).await?,
                context.load_task(&anchors[0].value, &anchors[1].value).await?,
            ]);
        }
        let mut records = Vec::with_capacity(anchors.len());
        for anchor in anchors {
            records.push(context.load(anchor.entity_name.as_deref(), &anchor.value).await?);
        }
        Ok(records)
    }
}

#[async_trait]
impl McpTool for ContextTool {
    fn name(&self) -> &'static str {
        "rekall_context"
    }

    fn description(&self) -> &'static str {
        texts::CONTEXT
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "Space-separated anchors, e.g. `project:vega task:report-builder`. The value is the record's label, \
                 lowercase and without spaces, never its title. A single anchor is valid and loads that record alone.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let raw = Arguments::of(arguments).required_string("anchors")?;
        let anchors = Anchor::parse_all(Some(&raw))?;
        match self.load(&anchors).await {
            Ok(records) => Ok(ContextRenderer.render(&records)),
            Err(RekallError::UnknownAnchor(message)) => Err(ToolError::Failure(message)),
            Err(RekallError::AmbiguousAnchor { message, .. }) => {
                Err(ToolError::Failure(format!("{message}. Qualify it as `entity:value`.")))
            }
            Err(other) => Err(ToolError::from(other)),
        }
    }
}
