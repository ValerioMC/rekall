use super::*;
use crate::connection::Secret;

fn destination(log_content: bool) -> TelemetryDestination {
    TelemetryDestination { host: "http://localhost:3000".into(), public_key: "pk-lf-one".into(), secret_key: Secret::new("sk-lf-two"), log_content }
}

#[test]
fn the_variables_point_claude_at_the_instance_with_basic_credentials() {
    let variables = launch_variables(&destination(false), "vega", Some("report-builder"));

    assert_eq!(variables["CLAUDE_CODE_ENABLE_TELEMETRY"], "1");
    assert_eq!(variables["CLAUDE_CODE_ENHANCED_TELEMETRY_BETA"], "1");
    assert_eq!(variables["OTEL_TRACES_EXPORTER"], "otlp");
    assert_eq!(variables["OTEL_EXPORTER_OTLP_PROTOCOL"], "http/protobuf");
    assert_eq!(variables["OTEL_EXPORTER_OTLP_ENDPOINT"], "http://localhost:3000/api/public/otel");
    assert_eq!(variables["OTEL_EXPORTER_OTLP_HEADERS"], "Authorization=Basic cGstbGYtb25lOnNrLWxmLXR3bw==");
}

#[test]
fn the_traces_carry_the_project_and_the_task_they_were_opened_on() {
    assert_eq!(launch_variables(&destination(false), "vega", Some("report-builder"))["OTEL_RESOURCE_ATTRIBUTES"], "rekall.project=vega,rekall.task=report-builder");
    assert_eq!(launch_variables(&destination(false), "vega", None)["OTEL_RESOURCE_ATTRIBUTES"], "rekall.project=vega");
}

#[test]
fn prompt_and_command_text_travel_only_when_the_destination_asks() {
    let redacted = launch_variables(&destination(false), "vega", None);
    assert!(!redacted.contains_key("OTEL_LOG_USER_PROMPTS"));
    assert!(!redacted.contains_key("OTEL_LOG_TOOL_DETAILS"));

    let shown = launch_variables(&destination(true), "vega", None);
    assert_eq!(shown["OTEL_LOG_USER_PROMPTS"], "1");
    assert_eq!(shown["OTEL_LOG_TOOL_DETAILS"], "1");
}
