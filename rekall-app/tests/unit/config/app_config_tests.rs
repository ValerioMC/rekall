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
fn a_database_url_names_a_sqlite_file_or_memory() {
    assert_eq!(DatabaseOverride::parse("sqlite::memory:"), Some(DatabaseOverride::Memory));
    assert_eq!(DatabaseOverride::parse("sqlite:/tmp/x.db"), Some(DatabaseOverride::File(PathBuf::from("/tmp/x.db"))));
    assert_eq!(DatabaseOverride::parse("sqlite:///tmp/x.db?mode=rwc"), Some(DatabaseOverride::File(PathBuf::from("/tmp/x.db"))));
    assert_eq!(DatabaseOverride::parse("/tmp/y.db"), Some(DatabaseOverride::File(PathBuf::from("/tmp/y.db"))));
    assert_eq!(DatabaseOverride::parse("  "), None);
}
