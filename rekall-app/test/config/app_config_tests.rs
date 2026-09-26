use super::*;

#[test]
fn arguments_beat_the_environment_which_is_read_in_its_relaxed_form() {
    let environment = HashMap::from([
        ("SERVER_PORT".to_string(), "9000".to_string()),
        ("REKALL_CLAUDE_CLIPATH".to_string(), "/opt/claude".to_string()),
        ("REKALL_BACKUP_ENABLED".to_string(), "false".to_string()),
    ]);
    let properties = Properties::new(&["--server.port=9100".to_string()], environment);
    let config = AppConfig::from_sources(&properties);
    assert_eq!(config.port, 9100);
    assert_eq!(config.claude.cli_path.as_deref(), Some("/opt/claude"));
    assert!(!config.backup.enabled);
    assert_eq!(config.backup.keep, 10);
}

#[test]
fn a_jdbc_url_from_the_java_build_names_the_sqlite_file_beside_the_h2_one() {
    assert_eq!(
        DatabaseOverride::parse("jdbc:h2:file:/data/demo/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1"),
        Some(DatabaseOverride::File(PathBuf::from("/data/demo/rekall.db")))
    );
    assert_eq!(DatabaseOverride::parse("jdbc:h2:mem:rekall;DB_CLOSE_DELAY=-1"), Some(DatabaseOverride::Memory));
    assert_eq!(DatabaseOverride::parse("sqlite:/tmp/x.db"), Some(DatabaseOverride::File(PathBuf::from("/tmp/x.db"))));
    assert_eq!(DatabaseOverride::parse("  "), None);
}
