#[derive(Clone, Debug)]
pub struct BackupConfig {
    /// `rekall.backup.enabled`: the scheduled backups. A backup on demand works either way.
    pub enabled: bool,
    /// `rekall.backup.interval-hours`.
    pub interval_hours: i64,
    /// `rekall.backup.keep`.
    pub keep: usize,
}
