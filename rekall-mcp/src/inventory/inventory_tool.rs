use async_trait::async_trait;
use rekall_core::inventory::DeclaredDependency;
use rekall_core::vulnerability::{InventoryRecorded, InventoryRecordingService};
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredProject, Arguments};

/// Lets a session write a project's dependency inventory whole. It replaces the list and settles
/// the exposures whose package is gone; it never asks the advisory database.
pub struct InventoryTool {
    recording: InventoryRecordingService,
}

impl InventoryTool {
    pub fn new(recording: InventoryRecordingService) -> Self {
        Self { recording }
    }
}

#[async_trait]
impl McpTool for InventoryTool {
    fn name(&self) -> &'static str {
        "rekall_inventory"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::INVENTORY
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string("anchors", "The project the inventory is for, `project:vega`. A label, never a title.")
            .required_object_array(
                "dependencies",
                "The whole list: `[{\"ecosystem\":\"npm\",\"name\":\"lodash\",\"version\":\"4.17.21\",\"manifest\":\"package-lock.json\"}]`, \
                 the exact version resolved and the declaring file relative to the project's folder.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredProject::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let declared = declared_of(args.value("dependencies"))?;
        let recorded = self.recording.record(&target.project_label, &declared).await.map_err(|e| told(e, ""))?;
        Ok(report(&recorded))
    }
}

/// The list as an array, or as a string holding one: some clients serialise a nested argument.
fn declared_of(argument: Option<&Value>) -> Result<Vec<DeclaredDependency>, ToolError> {
    let parsed = match argument {
        None => return Err(ToolError::Illegal("An inventory needs its `dependencies`.".into())),
        Some(Value::String(text)) => serde_json::from_str(text),
        Some(array) => serde_json::from_value(array.clone()),
    };
    parsed.map_err(|error| ToolError::Illegal(format!("'dependencies' is not a list of {{ecosystem, name, version, manifest}} objects: {error}")))
}

fn report(recorded: &InventoryRecorded) -> String {
    let written = &recorded.written;
    let ecosystems: Vec<String> = written.by_ecosystem.iter().map(|(name, count)| format!("{count} {name}")).collect();
    let repeats = match written.repeats {
        0 => String::new(),
        1 => " One entry repeated another and was kept once.".to_string(),
        many => format!(" {many} entries repeated others and were kept once."),
    };
    let settled = match recorded.settled_exposures {
        0 => String::new(),
        1 => " One exposure whose package is gone at that version is no longer a warning.".to_string(),
        many => format!(" {many} exposures whose package is gone at that version are no longer warnings."),
    };
    format!(
        "Inventory of `{}` recorded: {} dependencies ({}).{repeats}{settled} The next scan on the console's Security screen checks them against the critical advisories.",
        written.project_anchor,
        written.stored,
        ecosystems.join(", ")
    )
}

#[cfg(test)]
#[path = "../../tests/unit/inventory/inventory_tool_tests.rs"]
mod tests;
