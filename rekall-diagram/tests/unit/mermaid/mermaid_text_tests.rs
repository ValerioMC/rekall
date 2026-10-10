use super::*;

const SEQUENCE: &str = "---
title: a -> b
---
%%{init: {'theme': 'dark'}}%%
sequenceDiagram
  autonumber
  participant S as OrderService
  Note over S: a -> b is not a message
  U->>+S: place(order)
  S-->>P: charge
  alt declined
    P--xS: refused
  end
  S-)U: done
  A<<->>B: sync";

#[test]
fn the_declaration_comes_after_front_matter_comments_and_directives() {
    assert_eq!(MermaidText::new(SEQUENCE).declared_kind(), Some(DiagramKind::Sequence));
    assert_eq!(MermaidText::new("\n  flowchart LR\n A-->B").declared_kind(), Some(DiagramKind::Flowchart));
    assert_eq!(MermaidText::new("pie title Pets\n \"Dogs\": 3").declared_kind(), Some(DiagramKind::Pie));
}

#[test]
fn text_that_declares_nothing_known_has_no_kind() {
    assert_eq!(MermaidText::new("A --> B").declared_kind(), None);
    assert_eq!(MermaidText::new("%% only a comment").declared_kind(), None);
}

#[test]
fn every_arrow_is_a_message_and_notes_and_blocks_are_not() {
    assert_eq!(MermaidText::new(SEQUENCE).message_count(), 5);
}

#[test]
fn an_id_is_mentioned_only_as_a_whole_token_after_the_declaration() {
    let text = MermaidText::new("flowchart LR\n  validate[Validate order] --> charge");
    assert!(text.mentions("validate"));
    assert!(text.mentions("charge"));
    assert!(!text.mentions("valid"));
    assert!(!text.mentions("flowchart"));
}
