use std::collections::HashMap;

use rekall_kernel::Id;
use rekall_store::{project_section, section_context};

/// The pages in the order the sections rail shows them: contexts by their position, then pages by theirs.
pub fn in_project_order(contexts: &[section_context::Model], mut pages: Vec<project_section::Model>) -> Vec<project_section::Model> {
    let context_rank: HashMap<Id, usize> = contexts.iter().enumerate().map(|(rank, context)| (context.id, rank)).collect();
    pages.sort_by_key(|page| (context_rank.get(&page.context_id).copied().unwrap_or(usize::MAX), page.position, page.created_at));
    pages
}

#[cfg(test)]
#[path = "../../../tests/unit/export/rule/page_order_tests.rs"]
mod tests;
