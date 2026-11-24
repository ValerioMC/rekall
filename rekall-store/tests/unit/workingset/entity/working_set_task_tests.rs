use super::*;

#[test]
fn a_negative_position_is_refused() {
    let open = Model { task_id: Id::random(), position: -1 };
    assert!(open.validate().is_err());
}

#[test]
fn the_first_position_passes() {
    assert!(Model { task_id: Id::random(), position: 0 }.validate().is_ok());
}
