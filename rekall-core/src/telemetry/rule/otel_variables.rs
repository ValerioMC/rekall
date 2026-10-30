use std::collections::HashMap;

use base64::Engine;

use crate::telemetry::TelemetryDestination;

const OTLP_PATH: &str = "/api/public/otel";

/// The variables that make `claude` export its traces to the destination, tagged with the project
/// and task it was opened on. Prompt and command text is exported only when the destination asks.
pub fn launch_variables(destination: &TelemetryDestination, project_label: &str, task_label: Option<&str>) -> HashMap<String, String> {
    let mut variables = HashMap::from([
        ("CLAUDE_CODE_ENABLE_TELEMETRY".to_string(), "1".to_string()),
        ("CLAUDE_CODE_ENHANCED_TELEMETRY_BETA".to_string(), "1".to_string()),
        ("OTEL_TRACES_EXPORTER".to_string(), "otlp".to_string()),
        ("OTEL_EXPORTER_OTLP_PROTOCOL".to_string(), "http/protobuf".to_string()),
        ("OTEL_EXPORTER_OTLP_ENDPOINT".to_string(), format!("{}{OTLP_PATH}", destination.host)),
        ("OTEL_EXPORTER_OTLP_HEADERS".to_string(), format!("Authorization=Basic {}", basic_credentials(destination))),
        ("OTEL_RESOURCE_ATTRIBUTES".to_string(), resource_attributes(project_label, task_label)),
    ]);
    if destination.log_content {
        variables.insert("OTEL_LOG_USER_PROMPTS".to_string(), "1".to_string());
        variables.insert("OTEL_LOG_TOOL_DETAILS".to_string(), "1".to_string());
    }
    variables
}

fn basic_credentials(destination: &TelemetryDestination) -> String {
    let pair = format!("{}:{}", destination.public_key, destination.secret_key.expose());
    base64::engine::general_purpose::STANDARD.encode(pair)
}

fn resource_attributes(project_label: &str, task_label: Option<&str>) -> String {
    let mut attributes = vec![format!("rekall.project={project_label}")];
    attributes.extend(task_label.map(|label| format!("rekall.task={label}")));
    attributes.join(",")
}

#[cfg(test)]
#[path = "../../../tests/unit/telemetry/rule/otel_variables_tests.rs"]
mod tests;
