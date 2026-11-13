use async_trait::async_trait;
use rekall_kernel::RekallError;
use rekall_core::{context::AnchorEntity, context::ContextService, context::{ContextRecord, ContextRenderer}};
use serde_json::Value;

use crate::tool::{Anchor, Arguments};
use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;

pub struct ContextTool {
    context: ContextService,
}

impl ContextTool {
    pub fn new(context: ContextService) -> Self {
        Self { context }
    }

    async fn load(&self, anchors: &[Anchor]) -> Result<Vec<ContextRecord>, RekallError> {
        let context = &self.context;
        if anchors.len() == 2 && anchors[0].is("project") && anchors[1].is("task") {
            return Ok(vec![
                context.load(Some(AnchorEntity::Project), &anchors[0].value).await?,
                context.load_task(&anchors[0].value, &anchors[1].value).await?,
            ]);
        }
        let mut records = Vec::with_capacity(anchors.len());
        for anchor in anchors {
            let entity = anchor.entity_name.as_deref().map(AnchorEntity::parse).transpose()?;
            records.push(context.load(entity, &anchor.value).await?);
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
                "Space-separated anchors, e.g. `task:RK-12`. The value of a company or project is its label, a task's its code (`RK-12`), \
                 never its title. A single anchor is valid and loads that record alone.",
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
