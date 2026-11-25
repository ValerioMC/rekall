use rekall_store::ExportFormat;

const STEM_MAX_CHARACTERS: usize = 100;
const FALLBACK_STEM: &str = "export";

/// `<project label>-<what was exported>.<extension>`, lower-case and with safe characters only.
pub fn of_export(project_label: &str, what: Option<&str>, format: ExportFormat) -> String {
    let stem = what.map(safe_stem).filter(|stem| !stem.is_empty()).unwrap_or_else(|| FALLBACK_STEM.to_string());
    format!("{project_label}-{stem}.{}", extension_of(format))
}

pub fn media_type_of(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Pdf => "application/pdf",
        ExportFormat::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    }
}

/// What a person calls the file: the PDF, the Word file.
pub fn noun_of(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Pdf => "PDF",
        ExportFormat::Docx => "Word file",
    }
}

fn extension_of(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Pdf => "pdf",
        ExportFormat::Docx => "docx",
    }
}

fn safe_stem(title: &str) -> String {
    let dashed: String = title.chars().map(|character| if character.is_ascii_alphanumeric() { character.to_ascii_lowercase() } else { '-' }).collect();
    let words: Vec<&str> = dashed.split('-').filter(|word| !word.is_empty()).collect();
    words.join("-").chars().take(STEM_MAX_CHARACTERS).collect::<String>().trim_end_matches('-').to_string()
}

#[cfg(test)]
#[path = "../../../tests/unit/export/rule/export_file_name_tests.rs"]
mod tests;
