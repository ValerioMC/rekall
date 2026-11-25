use super::*;

#[test]
fn a_render_failure_says_what_could_not_be_written() {
    let failure = ExportFailure::rendering(ExportFormat::Docx, "a diagram image could not be read");

    assert_eq!(failure.kind, ExportFailureKind::Render);
    assert_eq!(failure.message, "The Word file could not be written: a diagram image could not be read");
}

#[test]
fn a_storage_failure_says_what_could_not_be_stored() {
    let failure = ExportFailure::storing(ExportFormat::Pdf, "disk full");

    assert_eq!(failure.kind, ExportFailureKind::Storage);
    assert_eq!(failure.message, "The PDF could not be stored: disk full");
}

#[test]
fn the_error_names_the_kind() {
    let error = ExportFailure::storing(ExportFormat::Pdf, "disk full").into_error();

    assert_eq!(error.message(), "Export: Storage error. The PDF could not be stored: disk full");
}

#[test]
fn a_long_cause_is_capped_to_the_column() {
    let failure = ExportFailure::rendering(ExportFormat::Pdf, "x".repeat(5_000));

    assert_eq!(jstr::len(&failure.message), FAILURE_MESSAGE_MAX_CHARACTERS);
    assert!(failure.message.ends_with('…'));
}
