use rekall_kernel::Instant;

use super::*;
use crate::section::ContextPlacementCommand;

fn context(project_id: Id) -> section_context::Model {
    let now = Instant::now();
    section_context::Model { id: Id::random(), project_id, title: "Operations".into(), position: 0, created_at: now, updated_at: now }
}

fn section_in(context: &section_context::Model) -> project_section::Model {
    let now = Instant::now();
    project_section::Model {
        id: Id::random(),
        project_id: context.project_id,
        context_id: context.id,
        title: "Deployment".into(),
        body_markdown: String::new(),
        position: 0,
        created_at: now,
        updated_at: now,
    }
}

fn placement(context: &section_context::Model, sections: &[&project_section::Model]) -> Placement {
    Placement { context_id: context.id, section_ids: sections.iter().map(|section| section.id).collect() }
}

#[test]
fn an_arrangement_naming_every_context_and_section_once_is_whole() {
    let project_id = Id::random();
    let (operations, team) = (context(project_id), context(project_id));
    let (deployment, glossary) = (section_in(&operations), section_in(&team));

    let swapped = [placement(&team, &[&glossary, &deployment]), placement(&operations, &[])];

    assert!(require_whole(&swapped, &[operations, team], &[deployment, glossary]).is_ok());
}

#[test]
fn a_section_left_out_or_named_twice_is_refused() {
    let operations = context(Id::random());
    let (deployment, glossary) = (section_in(&operations), section_in(&operations));
    let known = [deployment.clone(), glossary.clone()];

    for named in [vec![&deployment], vec![&deployment, &deployment], vec![&deployment, &glossary, &deployment]] {
        let refused = require_whole(&[placement(&operations, &named)], std::slice::from_ref(&operations), &known).unwrap_err();
        assert!(refused.message().contains("every section"), "{}", refused.message());
    }
}

#[test]
fn a_context_left_out_or_named_twice_is_refused() {
    let project_id = Id::random();
    let (operations, team) = (context(project_id), context(project_id));

    for named in [vec![placement(&operations, &[])], vec![placement(&operations, &[]), placement(&operations, &[])]] {
        let refused = require_whole(&named, &[operations.clone(), team.clone()], &[]).unwrap_err();
        assert!(refused.message().contains("every context"), "{}", refused.message());
    }
}

#[test]
fn an_arrangement_without_contexts_or_with_a_bad_id_is_refused() {
    assert!(placements_of(&SectionLayoutCommand { contexts: None }).is_err());
    let bad_context = SectionLayoutCommand { contexts: Some(vec![ContextPlacementCommand { context_id: Some("nope".into()), section_ids: Some(vec![]) }]) };
    assert!(placements_of(&bad_context).unwrap_err().message().contains("not a context id"));
    let bad_section = SectionLayoutCommand {
        contexts: Some(vec![ContextPlacementCommand { context_id: Some(Id::random().to_string()), section_ids: Some(vec!["nope".into()]) }]),
    };
    assert!(placements_of(&bad_section).unwrap_err().message().contains("not a section id"));
}
