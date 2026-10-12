use std::collections::{BTreeMap, HashSet};

use tracing::warn;

use super::{
    ClaudeHome, MemoryAnalysis, MemoryGroup, MemoryRef, ObsoleteMemory, PromotionCandidate, PromotionTarget, ReferenceLibrary,
    ResourceCatalog, ResourcePath, TextFingerprint,
};

/// A memory with fewer meaningful words than this says too little to compare.
const MIN_TERMS: usize = 6;
/// A memory is redundant when a document uses nearly all its words and repeats half its phrasing.
const COVERED_TERMS: f64 = 0.8;
const COVERED_PAIRS: f64 = 0.5;
/// Two memories say the same thing when this share of the shorter one's words is in the other,
/// and enough of their combined words is common that a long note does not match by size alone.
const SAME_TOPIC_OVERLAP: f64 = 0.45;
const SAME_TOPIC_SIMILARITY: f64 = 0.15;
const MIN_SHARED_TERMS: usize = 5;
const TOPIC_WORDS: usize = 6;
const SHARED_PHRASES: usize = 4;
/// Past this a memory is a procedure rather than a one-line rule.
const LONG_MEMORY_WORDS: usize = 150;

struct ReadMemory {
    reference: MemoryRef,
    fingerprint: TextFingerprint,
    words: usize,
    procedural: bool,
}

/// Reads every memory against the others, to find what keeps coming back, and against the
/// instructions and skills Claude loads, to find what they already say.
pub struct MemoryAnalyzer<'a> {
    home: &'a ClaudeHome,
}

impl<'a> MemoryAnalyzer<'a> {
    pub fn new(home: &'a ClaudeHome) -> Self {
        Self { home }
    }

    pub fn analyze(&self, catalog: &ResourceCatalog) -> MemoryAnalysis {
        let library = ReferenceLibrary::load(self.home, catalog);
        let memories: Vec<ReadMemory> = catalog.memory_groups.iter().flat_map(|group| self.read_group(group)).collect();
        let obsolete: Vec<ObsoleteMemory> = memories.iter().filter_map(|memory| covered(memory, &library)).collect();
        let redundant: HashSet<&str> = obsolete.iter().map(|finding| finding.memory.path.as_str()).collect();
        let worth_comparing: Vec<&ReadMemory> = memories.iter().filter(|memory| !redundant.contains(memory.reference.path.as_str())).collect();
        MemoryAnalysis {
            memories_read: memories.len(),
            documents_read: library.document_count(),
            promotions: recurring(&worth_comparing),
            obsolete: ranked(obsolete),
        }
    }

    fn read_group(&self, group: &MemoryGroup) -> Vec<ReadMemory> {
        group.memories.iter().filter_map(|summary| {
            let path = ResourcePath::parse(&summary.path).ok()?;
            let body = match self.home.read(&path) {
                Ok(body) => body,
                Err(failure) => {
                    warn!(%failure, path = %summary.path, "Left a memory out of the analysis");
                    return None;
                }
            };
            let fingerprint = TextFingerprint::of(&body);
            (fingerprint.term_count() >= MIN_TERMS).then(|| ReadMemory {
                reference: MemoryRef::of(summary, &group.id),
                fingerprint,
                words: body.split_whitespace().count(),
                procedural: body.contains("```") || body.lines().any(is_numbered_step),
            })
        }).collect()
    }
}

fn is_numbered_step(line: &str) -> bool {
    let trimmed = line.trim_start();
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && trimmed[digits..].starts_with(". ")
}

/// The document that repeats the most of a memory, when one repeats enough.
fn covered(memory: &ReadMemory, library: &ReferenceLibrary) -> Option<ObsoleteMemory> {
    let (document, pairs) = library
        .applying_to(&memory.reference.group_id)
        .filter(|document| memory.fingerprint.term_coverage_by(&document.fingerprint) >= COVERED_TERMS)
        .map(|document| (document, memory.fingerprint.pair_coverage_by(&document.fingerprint)))
        .filter(|(_, pairs)| *pairs >= COVERED_PAIRS)
        .max_by(|a, b| a.1.total_cmp(&b.1))?;
    Some(ObsoleteMemory {
        memory: memory.reference.clone(),
        covered_by: document.document.clone(),
        overlap_percent: (pairs * 100.0).round() as u8,
        shared_phrases: memory.fingerprint.shared_pairs(&document.fingerprint, SHARED_PHRASES),
    })
}

fn ranked(mut findings: Vec<ObsoleteMemory>) -> Vec<ObsoleteMemory> {
    findings.sort_by(|a, b| b.overlap_percent.cmp(&a.overlap_percent).then_with(|| a.memory.path.cmp(&b.memory.path)));
    findings
}

/// Groups memories that say the same thing and keeps the groups that span several projects: one
/// project repeating itself is a tidy-up, several is a rule.
fn recurring(memories: &[&ReadMemory]) -> Vec<PromotionCandidate> {
    let mut clusters = Clusters::new(memories.len());
    for (at, first) in memories.iter().enumerate() {
        for (later, second) in memories.iter().enumerate().skip(at + 1) {
            if say_the_same(first, second) {
                clusters.join(at, later);
            }
        }
    }
    let mut candidates: Vec<PromotionCandidate> =
        clusters.members().into_values().filter_map(|members| candidate_of(members.iter().map(|at| memories[*at]).collect())).collect();
    candidates.sort_by(|a, b| b.group_count.cmp(&a.group_count).then(b.memories.len().cmp(&a.memories.len())).then_with(|| a.topic.cmp(&b.topic)));
    candidates
}

fn say_the_same(first: &ReadMemory, second: &ReadMemory) -> bool {
    first.fingerprint.shared_term_count(&second.fingerprint) >= MIN_SHARED_TERMS
        && first.fingerprint.overlap(&second.fingerprint) >= SAME_TOPIC_OVERLAP
        && first.fingerprint.similarity(&second.fingerprint) >= SAME_TOPIC_SIMILARITY
}

fn candidate_of(members: Vec<&ReadMemory>) -> Option<PromotionCandidate> {
    let groups: HashSet<&str> = members.iter().map(|memory| memory.reference.group_id.as_str()).collect();
    if members.len() < 2 || groups.len() < 2 {
        return None;
    }
    let target = target_for(&members);
    let reason = match target {
        PromotionTarget::Instructions => "Short and the same everywhere: one line in the global CLAUDE.md would serve every project.",
        PromotionTarget::Skill => "Long or step by step: a skill Claude loads when it applies would serve every project.",
    };
    Some(PromotionCandidate {
        topic: topic_of(&members),
        memories: members.iter().map(|memory| memory.reference.clone()).collect(),
        group_count: groups.len(),
        target,
        reason: reason.to_string(),
    })
}

fn target_for(members: &[&ReadMemory]) -> PromotionTarget {
    let average = members.iter().map(|memory| memory.words).sum::<usize>() / members.len();
    if members.iter().any(|memory| memory.procedural) || average > LONG_MEMORY_WORDS {
        PromotionTarget::Skill
    } else {
        PromotionTarget::Instructions
    }
}

/// The words most of the memories use, which is what they are about.
fn topic_of(members: &[&ReadMemory]) -> Vec<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for term in members.iter().flat_map(|memory| memory.fingerprint.terms()) {
        *counts.entry(term).or_default() += 1;
    }
    let mut shared: Vec<(&str, usize)> = counts.into_iter().filter(|(_, count)| *count >= 2).collect();
    shared.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.len().cmp(&a.0.len())).then_with(|| a.0.cmp(b.0)));
    shared.into_iter().take(TOPIC_WORDS).map(|(term, _)| term.to_string()).collect()
}

/// Union-find over the positions of the memories being compared.
struct Clusters {
    parent: Vec<usize>,
}

impl Clusters {
    fn new(size: usize) -> Self {
        Self { parent: (0..size).collect() }
    }

    fn root(&mut self, at: usize) -> usize {
        if self.parent[at] != at {
            let root = self.root(self.parent[at]);
            self.parent[at] = root;
        }
        self.parent[at]
    }

    fn join(&mut self, first: usize, second: usize) {
        let (a, b) = (self.root(first), self.root(second));
        self.parent[a] = b;
    }

    fn members(mut self) -> BTreeMap<usize, Vec<usize>> {
        let mut by_root: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for at in 0..self.parent.len() {
            let root = self.root(at);
            by_root.entry(root).or_default().push(at);
        }
        by_root
    }
}

#[cfg(test)]
#[path = "../../tests/unit/resources/memory_analyzer_tests.rs"]
mod tests;
