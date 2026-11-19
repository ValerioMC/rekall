use super::*;

#[derive(Clone, Debug, PartialEq)]
struct Row {
    id: Id,
}

#[test]
fn the_record_with_the_id_is_returned() {
    let rows = [Row { id: Id::random() }, Row { id: Id::random() }];

    assert_eq!(require_by_id(&rows, rows[1].id, |row| row.id, "the rows were just read").unwrap(), rows[1]);
}

#[test]
fn a_missing_id_is_an_internal_error_stating_the_invariant() {
    let rows = [Row { id: Id::random() }];

    let failure = require_by_id(&rows, Id::random(), |row| row.id, "the order was checked against the rows").unwrap_err();

    assert!(matches!(&failure, RekallError::Internal(message) if message.contains("the order was checked against the rows")));
}
