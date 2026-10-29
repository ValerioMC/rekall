use super::*;

#[test]
fn a_budget_from_one_to_the_most_is_kept() {
    assert_eq!(VisitBudget::new(1).unwrap().visits(), 1);
    assert_eq!(VisitBudget::new(MOST).unwrap().visits(), MOST);
}

#[test]
fn no_visit_at_all_or_more_than_the_most_is_refused() {
    assert!(VisitBudget::new(0).unwrap_err().to_string().contains("runs from 1"));
    assert!(VisitBudget::new(MOST + 1).is_err());
}
