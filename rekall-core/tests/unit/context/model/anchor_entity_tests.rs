use super::*;

#[test]
fn every_alias_names_its_entity() {
    let aliases = [
        (AnchorEntity::Company, ["company", "companies", "azienda", "aziende", "cliente"].as_slice()),
        (AnchorEntity::Project, ["project", "projects", "progetto", "progetti"].as_slice()),
        (AnchorEntity::Task, ["task", "tasks", "issue", "issues", "attivita"].as_slice()),
        (AnchorEntity::Note, ["note", "notes", "nota", "document"].as_slice()),
    ];

    for (entity, names) in aliases {
        for name in names {
            assert_eq!(AnchorEntity::parse(name).unwrap(), entity, "{name}");
        }
    }
}

#[test]
fn the_letter_case_does_not_matter() {
    assert_eq!(AnchorEntity::parse("TASK").unwrap(), AnchorEntity::Task);
    assert_eq!(AnchorEntity::parse("Progetto").unwrap(), AnchorEntity::Project);
}

#[test]
fn an_unknown_entity_is_refused_with_the_entities_there_are() {
    let failure = AnchorEntity::parse("Sprint").unwrap_err();

    assert_eq!(failure.message(), "No entity matches 'Sprint'. Known entities: company, project, task, note");
}
