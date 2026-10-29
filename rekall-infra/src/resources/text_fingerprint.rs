use std::collections::HashSet;

const MIN_TERM_CHARS: usize = 3;

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "are", "but", "not", "you", "all", "any", "can", "had", "her", "was", "one", "our", "out", "has", "have", "from",
    "with", "this", "that", "they", "will", "would", "there", "their", "what", "when", "where", "which", "who", "whom", "whose", "why",
    "how", "also", "been", "being", "does", "doing", "done", "each", "else", "into", "just", "like", "more", "most", "much", "must", "only",
    "other", "over", "same", "should", "some", "such", "than", "then", "them", "these", "those", "too", "use", "used", "using", "very",
    "via", "were", "while", "without", "within", "about", "after", "again", "before", "because", "both", "could", "did", "its", "may",
    "own", "per", "see", "set", "get", "got", "now", "never", "always", "apply", "rule", "user", "claude", "della", "delle", "degli",
    "dei", "che", "con", "per", "non", "una", "uno", "gli", "nel", "nella", "sono", "come", "anche", "più", "perché", "quando", "dove",
];

/// The words of a text that carry its meaning, and the pairs of them that sit side by side:
/// enough to tell whether two texts say the same thing without understanding either.
#[derive(Clone, Debug, Default)]
pub struct TextFingerprint {
    terms: HashSet<String>,
    pairs: HashSet<String>,
}

impl TextFingerprint {
    pub fn of(text: &str) -> Self {
        let tokens: Vec<String> = without_front_matter(text).split(|c: char| !(c.is_alphanumeric() || c == '_')).filter_map(term_of).collect();
        let pairs = tokens.windows(2).map(|pair| format!("{} {}", pair[0], pair[1])).collect();
        Self { terms: tokens.into_iter().collect(), pairs }
    }

    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// The share of this text's terms that the other also uses.
    pub fn term_coverage_by(&self, other: &Self) -> f64 {
        share(self.terms.intersection(&other.terms).count(), self.terms.len())
    }

    /// The share of this text's word pairs that the other also has: what separates a rule restated
    /// from a document that merely uses the same vocabulary.
    pub fn pair_coverage_by(&self, other: &Self) -> f64 {
        share(self.pairs.intersection(&other.pairs).count(), self.pairs.len())
    }

    /// Shared terms over all terms, 0 for two texts with nothing in common.
    pub fn similarity(&self, other: &Self) -> f64 {
        share(self.terms.intersection(&other.terms).count(), self.terms.union(&other.terms).count())
    }

    pub fn shared_term_count(&self, other: &Self) -> usize {
        self.terms.intersection(&other.terms).count()
    }

    /// Shared terms over the smaller text's terms: a short rule restated inside a longer note
    /// still scores high.
    pub fn overlap(&self, other: &Self) -> f64 {
        share(self.shared_term_count(other), self.terms.len().min(other.terms.len()))
    }

    /// The word pairs both texts have, most telling first, for showing why two texts match.
    pub fn shared_pairs(&self, other: &Self, limit: usize) -> Vec<String> {
        let mut shared: Vec<String> = self.pairs.intersection(&other.pairs).cloned().collect();
        shared.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        shared.truncate(limit);
        shared
    }

    pub fn terms(&self) -> impl Iterator<Item = &str> {
        self.terms.iter().map(String::as_str)
    }
}

fn share(part: usize, whole: usize) -> f64 {
    if whole == 0 { 0.0 } else { part as f64 / whole as f64 }
}

fn term_of(word: &str) -> Option<String> {
    let lowered = word.trim_matches('_').to_lowercase();
    let stem = singular(&lowered);
    let meaningful = stem.chars().count() >= MIN_TERM_CHARS && !stem.chars().all(|c| c.is_ascii_digit()) && !STOPWORDS.contains(&stem);
    meaningful.then(|| stem.to_string())
}

/// Enough of a plural to match `dashes` with `dash` and `targets` with `target`.
fn singular(word: &str) -> &str {
    let sibilant_plural = word.len() > 5 && ["shes", "ches", "xes", "sses", "zes"].iter().any(|ending| word.ends_with(ending));
    if sibilant_plural {
        &word[..word.len() - 2]
    } else if word.len() > 4 && word.ends_with('s') && !["ss", "us", "is"].iter().any(|ending| word.ends_with(ending)) {
        &word[..word.len() - 1]
    } else {
        word
    }
}

fn without_front_matter(text: &str) -> &str {
    let trimmed = text.trim_start_matches('\u{feff}');
    match trimmed.strip_prefix("---") {
        Some(rest) => rest.split_once("\n---").map_or(trimmed, |(_, after)| after),
        None => trimmed,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/resources/text_fingerprint_tests.rs"]
mod tests;
