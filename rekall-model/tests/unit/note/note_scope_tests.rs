use super::*;

#[test]
fn the_columns_spell_the_scope_and_back() {
    let company = Id::random();
    let project = Id::random();
    for scope in [NoteScope::Global, NoteScope::Company { id: company }, NoteScope::Project { id: project }] {
        let (company_id, project_id) = scope.columns();
        assert_eq!(NoteScope::of_columns(company_id, project_id), scope);
    }
}

#[test]
fn a_scope_admits_only_tasks_inside_it() {
    let (acme, vega, rigel) = (Id::random(), Id::random(), Id::random());
    assert!(NoteScope::Global.admits(acme, vega));
    assert!(NoteScope::Company { id: acme }.admits(acme, rigel));
    assert!(!NoteScope::Company { id: Id::random() }.admits(acme, vega));
    assert!(NoteScope::Project { id: vega }.admits(acme, vega));
    assert!(!NoteScope::Project { id: vega }.admits(acme, rigel));
}

#[test]
fn the_narrowest_scope_is_the_one_project_then_the_one_company_then_global() {
    let (acme, globex, vega, rigel) = (Id::random(), Id::random(), Id::random(), Id::random());
    assert_eq!(NoteScope::narrowest(&[(acme, vega), (acme, vega)]), NoteScope::Project { id: vega });
    assert_eq!(NoteScope::narrowest(&[(acme, vega), (acme, rigel)]), NoteScope::Company { id: acme });
    assert_eq!(NoteScope::narrowest(&[(acme, vega), (globex, rigel)]), NoteScope::Global);
    assert_eq!(NoteScope::narrowest(&[]), NoteScope::Global);
}

#[test]
fn it_travels_as_a_kind_and_an_id() {
    let id = Id::random();
    let json = serde_json::to_value(NoteScope::Project { id }).unwrap();
    assert_eq!(json["kind"], "PROJECT");
    assert_eq!(serde_json::from_value::<NoteScope>(serde_json::json!({ "kind": "GLOBAL" })).unwrap(), NoteScope::Global);
}
