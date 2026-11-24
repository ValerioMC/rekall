use super::*;

fn ids(count: usize) -> Vec<Id> {
    (0..count).map(|_| Id::random()).collect()
}

#[test]
fn the_task_opened_after_the_closed_one_is_where_the_project_lands() {
    let open = ids(3);
    assert_eq!(landing_after_closing(&open, open[1]), Some(open[2]));
}

#[test]
fn the_last_task_closed_lands_on_the_one_before() {
    let open = ids(3);
    assert_eq!(landing_after_closing(&open, open[2]), Some(open[1]));
}

#[test]
fn the_only_task_closed_lands_on_the_task_list() {
    let open = ids(1);
    assert_eq!(landing_after_closing(&open, open[0]), None);
}
