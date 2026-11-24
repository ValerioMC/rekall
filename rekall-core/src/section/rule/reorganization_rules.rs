//! What a session's reorganization has to be before any page moves: every page named once, by a
//! title the project has, under a context with a title.

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::project_section;

use crate::section::rule::context_rules;
use crate::section::PageGroup;

/// One context the session wants, with the ids of its pages in order.
#[derive(Clone, Debug)]
pub(crate) struct WantedGroup {
    pub context: String,
    pub page_ids: Vec<Id>,
}

pub(crate) fn checked(groups: &[PageGroup], sections: &[project_section::Model]) -> Result<Vec<WantedGroup>> {
    if groups.is_empty() {
        return Err(RekallError::illegal("A reorganization names at least one context."));
    }
    let wanted = groups.iter().map(|group| resolved(group, sections)).collect::<Result<Vec<_>>>()?;
    require_each_context_once(&wanted)?;
    require_every_page_once(&wanted, sections)?;
    Ok(wanted)
}

fn resolved(group: &PageGroup, sections: &[project_section::Model]) -> Result<WantedGroup> {
    let context = context_rules::title(Some(&group.context))?;
    let page_ids = group
        .pages
        .iter()
        .map(|title| {
            sections
                .iter()
                .find(|section| jstr::equals_ignore_case(jstr::strip(&section.title), jstr::strip(title)))
                .map(|section| section.id)
                .ok_or_else(|| RekallError::illegal(format!("This project has no page titled '{}'. Nothing was moved.", jstr::strip(title))))
        })
        .collect::<Result<Vec<Id>>>()?;
    Ok(WantedGroup { context, page_ids })
}

fn require_each_context_once(wanted: &[WantedGroup]) -> Result<()> {
    for (index, group) in wanted.iter().enumerate() {
        let twice = wanted[..index].iter().any(|earlier| jstr::equals_ignore_case(&earlier.context, &group.context));
        if twice {
            return Err(RekallError::illegal(format!("The context '{}' is named twice. Nothing was moved.", group.context)));
        }
    }
    Ok(())
}

fn require_every_page_once(wanted: &[WantedGroup], sections: &[project_section::Model]) -> Result<()> {
    let placed: Vec<Id> = wanted.iter().flat_map(|group| group.page_ids.iter().copied()).collect();
    if let Some(twice) = placed.iter().enumerate().find(|(index, id)| placed[..*index].contains(id)).map(|(_, id)| *id) {
        let title = sections.iter().find(|section| section.id == twice).map(|section| section.title.as_str()).unwrap_or_default();
        return Err(RekallError::illegal(format!("The page '{title}' is named twice. Nothing was moved.")));
    }
    let left_out: Vec<&str> = sections.iter().filter(|section| !placed.contains(&section.id)).map(|section| section.title.as_str()).collect();
    if left_out.is_empty() {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "Every page has to be placed in a context, and these were not: {}. Nothing was moved.",
        left_out.join(", ")
    )))
}

#[cfg(test)]
#[path = "../../../tests/unit/section/rule/reorganization_rules_tests.rs"]
mod tests;
