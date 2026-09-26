use super::*;

#[test]
fn csvwrite_output_reads_back_with_nulls_empty_strings_quotes_and_line_breaks() {
    let rows = parse_csv("\"ID\",\"NAME\",\"NOTE\"\n\"1\",\"say \"\"hi\"\"\",\n\"2\",\"\",\"two\nlines\"\n");
    assert_eq!(rows[0], vec![Some("ID".into()), Some("NAME".into()), Some("NOTE".into())]);
    assert_eq!(rows[1], vec![Some("1".into()), Some("say \"hi\"".into()), None]);
    assert_eq!(rows[2], vec![Some("2".into()), Some(String::new()), Some("two\nlines".into())]);
    assert_eq!(rows.len(), 3);
}

#[test]
fn h2_values_take_the_spelling_the_sqlite_schema_stores() {
    assert_eq!(convert(Some("TRUE".into()), "BOOLEAN").unwrap(), Value::BigInt(Some(1)));
    assert_eq!(convert(None, "BOOLEAN").unwrap(), Value::String(None));
    assert_eq!(
        convert(Some("2026-03-01 10:15:30.5+02".into()), "TIMESTAMP").unwrap(),
        Value::String(Some("2026-03-01T08:15:30.500000Z".into()))
    );
    assert_eq!(
        convert(Some("2026-03-01 10:15:30+05:30".into()), "TIMESTAMP WITH TIME ZONE").unwrap(),
        Value::String(Some("2026-03-01T04:45:30.000000Z".into()))
    );
    assert_eq!(
        convert(Some("0F8FAD5B-D9CB-469F-A165-70867728950E".into()), "UUID").unwrap(),
        Value::String(Some("0f8fad5b-d9cb-469f-a165-70867728950e".into()))
    );
    assert_eq!(convert(Some("42".into()), "INTEGER").unwrap(), Value::BigInt(Some(42)));
}

#[test]
fn the_h2_file_beside_a_missing_sqlite_one_is_found() {
    let folder = tempfile::tempdir().unwrap();
    assert_eq!(legacy_file_beside(&folder.path().join("rekall.db")), None);
    std::fs::write(folder.path().join("rekall.mv.db"), "H:2").unwrap();
    assert_eq!(legacy_file_beside(&folder.path().join("rekall.db")), Some(folder.path().join("rekall.mv.db")));
}
