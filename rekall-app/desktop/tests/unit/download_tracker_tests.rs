use super::*;

const URL: &str = "http://127.0.0.1:47355/api/backups/rekall-20261004-101500-manual.zip";

#[test]
fn a_finished_download_without_a_path_falls_back_to_where_it_was_saved() {
    let tracker = DownloadTracker::default();
    tracker.started(URL, Path::new("/Users/me/Downloads/rekall-2026-10-04.zip"));
    assert_eq!(tracker.finished(URL, None), Some(PathBuf::from("/Users/me/Downloads/rekall-2026-10-04.zip")));
}

#[test]
fn a_path_the_webview_reports_wins_over_the_recorded_one() {
    let tracker = DownloadTracker::default();
    tracker.started(URL, Path::new("/a.zip"));
    assert_eq!(tracker.finished(URL, Some(PathBuf::from("/b.zip"))), Some(PathBuf::from("/b.zip")));
}

#[test]
fn the_same_url_downloaded_twice_resolves_to_the_second_destination() {
    let tracker = DownloadTracker::default();
    tracker.started(URL, Path::new("/first.zip"));
    tracker.finished(URL, None);
    tracker.started(URL, Path::new("/second.zip"));
    assert_eq!(tracker.finished(URL, None), Some(PathBuf::from("/second.zip")));
}

#[test]
fn a_download_that_never_started_has_no_path() {
    assert_eq!(DownloadTracker::default().finished(URL, None), None);
}
