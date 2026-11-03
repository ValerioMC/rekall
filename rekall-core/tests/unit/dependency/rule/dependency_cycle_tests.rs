use super::*;

fn ids() -> (Id, Id, Id) {
    (Id::random(), Id::random(), Id::random())
}

#[test]
fn a_task_cannot_depend_on_itself() {
    let (a, _, _) = ids();

    assert!(closes_a_cycle(&[], a, a));
}

#[test]
fn a_dependency_back_to_the_task_closes_a_cycle() {
    let (a, b, _) = ids();

    assert!(closes_a_cycle(&[(b, a)], a, b));
}

#[test]
fn a_cycle_through_several_tasks_is_found() {
    let (a, b, c) = ids();

    assert!(closes_a_cycle(&[(b, c), (c, a)], a, b));
}

#[test]
fn two_tasks_depending_on_a_third_make_no_cycle() {
    let (a, b, c) = ids();

    assert!(!closes_a_cycle(&[(a, c)], b, c));
}

#[test]
fn a_chain_in_one_direction_makes_no_cycle() {
    let (a, b, c) = ids();

    assert!(!closes_a_cycle(&[(b, c)], a, b));
}
