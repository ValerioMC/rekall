use std::collections::HashMap;

use rekall_common::Result;
use rekall_diagram::{GraphViolation, GraphViolations, SemanticGraph, SourceLocation};

use super::ProjectFolder;

/// Source files larger than this are not what a span points into.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
/// The same cap the graph validator puts on its own report.
const MAX_REPORTED: usize = 50;

/// Holds a generated graph's traceability to the code on disk: every cited file has to be in the
/// project folder, and every cited line inside that file. The validator cannot see the disk, so a
/// session that guessed a path or a line range is caught here rather than in the console.
pub struct SourceAudit<'a> {
    folder: &'a ProjectFolder,
    line_counts: HashMap<String, std::result::Result<u32, String>>,
    violations: Vec<GraphViolation>,
}

impl<'a> SourceAudit<'a> {
    pub fn check(folder: &'a ProjectFolder, graph: &SemanticGraph) -> std::result::Result<(), GraphViolations> {
        let mut audit = Self { folder, line_counts: HashMap::new(), violations: Vec::new() };
        for (node_index, node) in graph.nodes.iter().enumerate() {
            for (source_index, source) in node.sources.iter().enumerate() {
                audit.check_source(&format!("nodes[{node_index}].sources[{source_index}]"), source);
            }
        }
        if audit.violations.is_empty() {
            return Ok(());
        }
        audit.violations.truncate(MAX_REPORTED);
        Err(GraphViolations(audit.violations))
    }

    fn check_source(&mut self, path: &str, source: &SourceLocation) {
        let total = match self.line_count(&source.file) {
            Ok(total) => total,
            Err(reason) => return self.refuse(&format!("{path}.file"), &reason),
        };
        for (field, line) in [("startLine", source.start_line), ("endLine", source.end_line)] {
            if let Some(line) = line.filter(|line| *line > total) {
                self.refuse(&format!("{path}.{field}"), &format!("line {line} is past the end of {} ({total} lines)", source.file));
            }
        }
    }

    fn line_count(&mut self, file: &str) -> std::result::Result<u32, String> {
        let key = SourceLocation::normalise(file);
        if let Some(known) = self.line_counts.get(&key) {
            return known.clone();
        }
        let counted = count_lines(self.folder, &key).map_err(|error| error.to_string());
        self.line_counts.insert(key, counted.clone());
        counted
    }

    fn refuse(&mut self, path: &str, message: &str) {
        self.violations.push(GraphViolation { path: path.to_string(), message: message.to_string() });
    }
}

fn count_lines(folder: &ProjectFolder, file: &str) -> Result<u32> {
    let text = folder.read(file, MAX_FILE_BYTES)?;
    Ok(u32::try_from(text.lines().count()).unwrap_or(u32::MAX))
}

#[cfg(test)]
#[path = "../../tests/unit/diagram/source_audit_tests.rs"]
mod tests;
