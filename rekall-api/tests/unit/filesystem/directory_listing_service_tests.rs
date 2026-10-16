use super::*;

struct Fixture {
    home: tempfile::TempDir,
    service: DirectoryListingService,
}

impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let project = home.path().join("project");
        std::fs::create_dir_all(project.join("src/components")).unwrap();
        std::fs::create_dir_all(project.join("Docs")).unwrap();
        std::fs::write(project.join("README.md"), "#").unwrap();
        std::fs::write(project.join("app.ts"), "").unwrap();
        std::fs::write(project.join(".env.local"), "").unwrap();
        let service = DirectoryListingService::new(home.path());
        Self { home, service }
    }

    fn at(&self, relative: &str) -> PathBuf {
        let home = normalize(&absolute(self.home.path()));
        if relative.is_empty() {
            home
        } else {
            home.join(relative)
        }
    }

    fn path(&self, relative: &str) -> String {
        text(&self.at(relative))
    }
}

#[test]
fn folders_come_first_then_files_each_by_name_ignoring_case_hidden_ones_flagged() {
    let fixture = Fixture::new();
    let listing = fixture.service.list(Some(&fixture.path("project")), None).unwrap();

    let names: Vec<&str> = listing.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Docs", "src", ".env.local", "app.ts", "README.md"]);
    let hidden: Vec<&str> = listing.entries.iter().filter(|e| e.hidden).map(|e| e.name.as_str()).collect();
    assert_eq!(hidden, [".env.local"]);
    assert_eq!(listing.entries[0].path, fixture.path("project/Docs"));
    assert!(listing.readable);
    assert!(!listing.truncated);
}

#[test]
fn a_typed_path_is_taken_relative_to_the_folder_the_picker_stands_in_and_normalised() {
    let fixture = Fixture::new();
    let base = fixture.path("project");

    assert_eq!(
        fixture.service.list(Some(&base), Some("src/components/../components")).unwrap().path,
        fixture.path("project/src/components")
    );
    assert_eq!(fixture.service.list(Some(&base), Some("..")).unwrap().path, fixture.path(""));
}

#[test]
fn tilde_is_the_home_folder_and_an_absolute_path_ignores_the_base() {
    let fixture = Fixture::new();
    let base = fixture.path("project/src");

    assert_eq!(fixture.service.list(Some(&base), Some("~")).unwrap().path, fixture.path(""));
    assert_eq!(fixture.service.list(Some(&base), Some("~/project")).unwrap().path, fixture.path("project"));
    assert_eq!(
        fixture.service.list(Some(&base), Some(&fixture.path("project/Docs"))).unwrap().path,
        fixture.path("project/Docs")
    );
}

#[test]
fn with_no_base_and_no_path_the_listing_is_the_home_folder() {
    let fixture = Fixture::new();
    let listing = fixture.service.list(None, Some("  ")).unwrap();

    assert_eq!(listing.path, fixture.path(""));
    assert_eq!(listing.home, fixture.path(""));
}

#[test]
fn the_breadcrumb_runs_from_the_root_to_the_folder_each_crumb_carrying_its_own_path() {
    let fixture = Fixture::new();
    let folder = fixture.at("project/src");

    let listing = fixture.service.list(Some(&text(&folder)), None).unwrap();

    let root: PathBuf = folder.ancestors().last().unwrap().to_path_buf();
    assert_eq!(listing.segments.first().unwrap().path, text(&root));
    assert_eq!(listing.segments.last().unwrap(), &PathSegmentResponse { name: "src".into(), path: text(&folder) });
    assert_eq!(listing.parent, Some(fixture.path("project")));
}

#[cfg(unix)]
#[test]
fn the_root_has_no_parent_and_a_dot_dot_above_it_stays_there() {
    assert_eq!(normalize(Path::new("/..")), PathBuf::from("/"));
    assert_eq!(normalize(Path::new("/a/./b/../c")), PathBuf::from("/a/c"));
    let listing = DirectoryListingService::new("/").list(Some("/"), None).unwrap();
    assert_eq!(listing.parent, None);
    assert_eq!(listing.segments, [PathSegmentResponse { name: "/".into(), path: "/".into() }]);
}

#[test]
fn a_missing_folder_is_not_found_and_a_file_is_refused_as_a_folder() {
    let fixture = Fixture::new();
    let base = fixture.path("project");

    assert!(matches!(fixture.service.list(Some(&base), Some("nope")), Err(RekallError::NotFound(_))));
    match fixture.service.list(Some(&base), Some("README.md")) {
        Err(RekallError::IllegalArgument(message)) => assert!(message.contains("is a file")),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[test]
fn a_folder_bigger_than_the_limit_is_cut_after_sorting_and_flagged() {
    let fixture = Fixture::new();
    let crowded = fixture.at("crowded");
    std::fs::create_dir_all(&crowded).unwrap();
    for index in 0..=ENTRY_LIMIT {
        std::fs::write(crowded.join(format!("file-{index:05}")), "").unwrap();
    }

    let listing = fixture.service.list(Some(&text(&crowded)), None).unwrap();

    assert!(listing.truncated);
    assert_eq!(listing.entries.len(), ENTRY_LIMIT);
    assert_eq!(listing.entries.last().unwrap().name, format!("file-{:05}", ENTRY_LIMIT - 1));
}
