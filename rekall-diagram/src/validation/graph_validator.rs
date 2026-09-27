use std::collections::{HashMap, HashSet};

use regex::Regex;

use crate::graph::{GraphNode, NodeKind, RelationKind, SemanticGraph, SourceLocation};

use super::{GraphViolation, GraphViolations};

pub const MAX_NODES: usize = 600;
pub const MAX_EDGES: usize = 2400;
const MAX_ID_CHARACTERS: usize = 120;
const MAX_TITLE_CHARACTERS: usize = 200;
const MAX_DESCRIPTION_CHARACTERS: usize = 4000;
const MAX_REPORTED: usize = 50;

/// Checks a parsed graph against the rules the console and the trace index rely on: unique ids,
/// edges between existing nodes, `contains` forming a forest, sane spans and confidences.
pub struct GraphValidator {
    violations: Vec<GraphViolation>,
    custom_name: Regex,
}

impl GraphValidator {
    pub fn validate(graph: &SemanticGraph) -> Result<(), GraphViolations> {
        let mut validator = Self { violations: Vec::new(), custom_name: Regex::new(r"^[a-z][a-z0-9_]{0,39}$").expect("valid pattern") };
        validator.check(graph);
        if validator.violations.is_empty() {
            return Ok(());
        }
        validator.violations.truncate(MAX_REPORTED);
        Err(GraphViolations(validator.violations))
    }

    fn check(&mut self, graph: &SemanticGraph) {
        if graph.nodes.is_empty() {
            self.refuse("nodes", "a graph needs at least one node");
        }
        if graph.nodes.len() > MAX_NODES {
            self.refuse("nodes", &format!("at most {MAX_NODES} nodes, found {}", graph.nodes.len()));
        }
        if graph.edges.len() > MAX_EDGES {
            self.refuse("edges", &format!("at most {MAX_EDGES} edges, found {}", graph.edges.len()));
        }
        let mut seen: HashSet<&str> = HashSet::new();
        for (index, node) in graph.nodes.iter().enumerate() {
            self.check_node(index, node);
            if !node.id.is_empty() && !seen.insert(node.id.as_str()) {
                self.refuse(&format!("nodes[{index}].id"), &format!("\"{}\" is used by an earlier node", node.id));
            }
        }
        self.check_edges(graph, &seen);
        self.check_containment(graph);
    }

    fn check_node(&mut self, index: usize, node: &GraphNode) {
        let at = format!("nodes[{index}]");
        self.check_id(&format!("{at}.id"), &node.id);
        if let NodeKind::Custom(name) = &node.kind {
            self.check_custom_name(&format!("{at}.kind"), name);
        }
        if node.title.is_empty() {
            self.refuse(&format!("{at}.title"), "must not be blank");
        }
        self.check_length(&format!("{at}.title"), &node.title, MAX_TITLE_CHARACTERS);
        if let Some(description) = &node.description {
            self.check_length(&format!("{at}.description"), description, MAX_DESCRIPTION_CHARACTERS);
        }
        self.check_confidence(&format!("{at}.confidence"), node.confidence);
        for (position, source) in node.sources.iter().enumerate() {
            self.check_source(&format!("{at}.sources[{position}]"), source);
        }
    }

    fn check_edges(&mut self, graph: &SemanticGraph, node_ids: &HashSet<&str>) {
        let mut edge_ids: HashSet<&str> = HashSet::new();
        let mut identical: HashSet<(&str, &str, &RelationKind, Option<&str>)> = HashSet::new();
        for (index, edge) in graph.edges.iter().enumerate() {
            let at = format!("edges[{index}]");
            self.check_id(&format!("{at}.id"), &edge.id);
            if !edge_ids.insert(edge.id.as_str()) {
                self.refuse(&format!("{at}.id"), &format!("\"{}\" is used by an earlier edge", edge.id));
            }
            self.check_endpoint(&format!("{at}.from"), &edge.from, node_ids);
            self.check_endpoint(&format!("{at}.to"), &edge.to, node_ids);
            if let RelationKind::Custom(name) = &edge.relation {
                self.check_custom_name(&format!("{at}.relation"), name);
            }
            self.check_confidence(&format!("{at}.confidence"), edge.confidence);
            if !identical.insert((edge.from.as_str(), edge.to.as_str(), &edge.relation, edge.label.as_deref())) {
                self.refuse(&at, "repeats an earlier edge with the same ends, relation and label");
            }
        }
    }

    fn check_endpoint(&mut self, path: &str, id: &str, node_ids: &HashSet<&str>) {
        if !node_ids.contains(id) {
            self.refuse(path, &format!("no node has the id \"{id}\""));
        }
    }

    /// `contains` has to be a forest: nothing contains itself, nothing has two containers, and
    /// following containers upward always ends.
    fn check_containment(&mut self, graph: &SemanticGraph) {
        let mut container: HashMap<&str, &str> = HashMap::new();
        for (index, edge) in graph.edges.iter().enumerate().filter(|(_, edge)| edge.relation == RelationKind::Contains) {
            if edge.from == edge.to {
                self.refuse(&format!("edges[{index}]"), "a node cannot contain itself");
            } else if let Some(existing) = container.insert(edge.to.as_str(), edge.from.as_str()) {
                self.refuse(&format!("edges[{index}]"), &format!("\"{}\" is already contained by \"{existing}\"", edge.to));
            }
        }
        for start in container.keys() {
            if Self::climbs_back(&container, start) {
                self.refuse("edges", &format!("\"contains\" runs in a cycle through \"{start}\""));
                return;
            }
        }
    }

    fn climbs_back(container: &HashMap<&str, &str>, start: &str) -> bool {
        let mut current = start;
        for _ in 0..=container.len() {
            match container.get(current) {
                Some(&parent) if parent == start => return true,
                Some(&parent) => current = parent,
                None => return false,
            }
        }
        true
    }

    fn check_source(&mut self, at: &str, source: &SourceLocation) {
        if source.file.trim().is_empty() {
            self.refuse(&format!("{at}.file"), "must not be blank");
        }
        match (source.start_line, source.end_line) {
            (Some(0), _) => self.refuse(&format!("{at}.startLine"), "lines are counted from 1"),
            (None, Some(_)) => self.refuse(&format!("{at}.endLine"), "an end line needs a start line"),
            (Some(start), Some(end)) if end < start => self.refuse(&format!("{at}.endLine"), "ends before it starts"),
            _ => {}
        }
    }

    fn check_id(&mut self, path: &str, id: &str) {
        if id.is_empty() {
            self.refuse(path, "must not be blank");
        } else if id.chars().any(char::is_whitespace) {
            self.refuse(path, "must not contain whitespace");
        }
        self.check_length(path, id, MAX_ID_CHARACTERS);
    }

    fn check_custom_name(&mut self, path: &str, name: &str) {
        if !self.custom_name.is_match(name) {
            self.refuse(path, &format!("\"{name}\" is neither a known kind nor a snake_case name"));
        }
    }

    fn check_confidence(&mut self, path: &str, confidence: Option<f64>) {
        if confidence.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
            self.refuse(path, "must be between 0 and 1");
        }
    }

    fn check_length(&mut self, path: &str, text: &str, max: usize) {
        if text.chars().count() > max {
            self.refuse(path, &format!("at most {max} characters"));
        }
    }

    fn refuse(&mut self, path: &str, message: &str) {
        self.violations.push(GraphViolation { path: path.to_string(), message: message.to_string() });
    }
}


#[cfg(test)]
#[path = "../../tests/unit/validation/graph_validator_tests.rs"]
mod tests;
