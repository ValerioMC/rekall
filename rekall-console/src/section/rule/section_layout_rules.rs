//! What an arrangement has to be before it is applied: it names every context of the project and
//! every section of it, each once, so a page is never lost or doubled by a drag.

use std::collections::HashSet;

use rekall_core::section::rule::placement_rules::Placement;
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{project_section, section_context};

use crate::section::SectionLayoutCommand;

pub(in crate::section) fn placements_of(command: &SectionLayoutCommand) -> Result<Vec<Placement>> {
    let contexts = command.contexts.as_ref().ok_or_else(|| RekallError::illegal("An arrangement names the contexts of the project."))?;
    contexts
        .iter()
        .map(|placement| {
            let raw_context = placement.context_id.as_deref().unwrap_or_default();
            let context_id = raw_context.parse::<Id>().map_err(|_| RekallError::illegal(format!("'{raw_context}' is not a context id.")))?;
            let section_ids = placement
                .section_ids
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|raw| raw.parse::<Id>().map_err(|_| RekallError::illegal(format!("'{raw}' is not a section id."))))
                .collect::<Result<Vec<Id>>>()?;
            Ok(Placement { context_id, section_ids })
        })
        .collect()
}

pub(in crate::section) fn require_whole(
    placements: &[Placement],
    contexts: &[section_context::Model],
    sections: &[project_section::Model],
) -> Result<()> {
    let placed_contexts: Vec<Id> = placements.iter().map(|placement| placement.context_id).collect();
    let placed_sections: Vec<Id> = placements.iter().flat_map(|placement| placement.section_ids.iter().copied()).collect();
    let known_contexts: Vec<Id> = contexts.iter().map(|context| context.id).collect();
    let known_sections: Vec<Id> = sections.iter().map(|section| section.id).collect();
    if !names_each_once(&placed_contexts, &known_contexts) {
        return Err(RekallError::illegal("An arrangement has to name every context of the project exactly once."));
    }
    if !names_each_once(&placed_sections, &known_sections) {
        return Err(RekallError::illegal("An arrangement has to name every section of the project exactly once."));
    }
    Ok(())
}

fn names_each_once(named: &[Id], known: &[Id]) -> bool {
    named.len() == known.len() && named.iter().collect::<HashSet<_>>() == known.iter().collect::<HashSet<_>>()
}

#[cfg(test)]
#[path = "../../../tests/unit/section/rule/section_layout_rules_tests.rs"]
mod tests;
