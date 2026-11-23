use super::*;
use crate::decision::{DecisionDetailView, DecisionOptionView, DecisionView};
use rekall_kernel::{Id, Instant};
use rekall_store::{DecisionSource, DecisionState};

fn decision(key: &str, state: DecisionState) -> DecisionView {
    DecisionView {
        id: Id::random(), key: key.into(), topic: None, question: format!("Question of {key}?"), why: None, step_number: None, step_title: None, state,
        answer: None, hold_note: None, previous_answer: None, changed_at: None, source: DecisionSource::Terminal, options: Vec::new(), lines: Vec::new(), details: Vec::new(),
    }
}

fn answered(key: &str, answer: &str) -> DecisionView {
    DecisionView { answer: Some(answer.into()), ..decision(key, DecisionState::Answered) }
}

fn on_step(number: i32, title: &str, view: DecisionView) -> DecisionView {
    DecisionView { step_number: Some(number), step_title: Some(title.into()), ..view }
}

#[test]
fn a_task_without_decisions_renders_nothing() {
    assert_eq!(ContextRenderer.render_decisions(&[]), "");
}

#[test]
fn each_record_is_one_line_in_the_form_its_state_reads_best() {
    let at = Instant::parse("2026-11-23T14:31:00Z").unwrap();
    let changed = DecisionView {
        previous_answer: Some("2".into()),
        details: vec![DecisionDetailView { text: "Autoscale\nbetween 3 and 6.".into(), added_at: at }],
        ..answered("replicas", "3")
    };
    let choosing = DecisionView {
        options: vec![DecisionOptionView { label: "Secret".into(), effect: None, recommended: false }, DecisionOptionView { label: "Vault".into(), effect: None, recommended: true }],
        ..decision("db-password", DecisionState::Open)
    };
    let on_hold = DecisionView { hold_note: Some("Asking the platform team".into()), ..decision("vault-path", DecisionState::OnHold) };
    let by_hand = DecisionView { source: DecisionSource::Person, ..answered("namespace", "billing") };

    let rendered = ContextRenderer.render_decisions(&[changed, choosing, decision("region", DecisionState::Open), on_hold, by_hand]);

    assert!(rendered.contains("<decisions count=\"5\" answered=\"2\" open=\"2\" on-hold=\"1\">"), "{rendered}");
    assert!(rendered.contains("\nreplicas: 3 (was 2) + 2026-11-23: Autoscale between 3 and 6.\n"), "{rendered}");
    assert!(rendered.contains("\ndb-password: ? Secret | Vault\n"), "{rendered}");
    assert!(rendered.contains("\nregion: ? Question of region?\n"), "{rendered}");
    assert!(rendered.contains("\nvault-path: on hold: Asking the platform team\n"), "{rendered}");
    assert!(rendered.contains("\nnamespace: billing [edited by you]\n"), "{rendered}");
}

#[test]
fn the_tasks_own_records_come_first_and_each_step_has_its_group() {
    let rendered = ContextRenderer.render_decisions(&[
        on_step(3, "Wire \"the\" ingress", answered("host", "billing.acme.internal")),
        on_step(2, "Scale", answered("replicas", "3")),
        answered("namespace", "billing"),
        on_step(3, "Wire \"the\" ingress", answered("tls", "yes")),
    ]);

    let expected = "<decisions count=\"4\" answered=\"4\" open=\"0\" on-hold=\"0\">\nnamespace: billing\n<step n=\"2\" title=\"Scale\">\nreplicas: 3\n</step>\n<step n=\"3\" title=\"Wire 'the' ingress\">\nhost: billing.acme.internal\ntls: yes\n</step>\n</decisions>";
    assert!(rendered.contains(expected), "{rendered}");
}
