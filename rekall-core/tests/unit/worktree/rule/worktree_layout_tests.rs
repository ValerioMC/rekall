use super::*;

fn layout() -> WorktreeLayout {
    WorktreeLayout::new(PathBuf::from("/data/worktrees"))
}

#[test]
fn a_tasks_branch_is_its_label_under_rekall() {
    assert_eq!(layout().branch_for("review-features").unwrap(), "rekall/review-features");
}

#[test]
fn a_tasks_folder_is_under_the_root_by_project_and_task() {
    assert_eq!(layout().folder_for("vega", "report").unwrap(), PathBuf::from("/data/worktrees/vega/report"));
}

#[test]
fn a_label_that_could_leave_the_folder_or_break_a_branch_name_is_refused() {
    for label in ["", "..", "a/b", "a..b", ".hidden", "-flag", "Upper", "with space", "a;b"] {
        assert!(layout().branch_for(label).is_err(), "{label}");
        assert!(layout().folder_for("vega", label).is_err(), "{label}");
        assert!(layout().folder_for(label, "report").is_err(), "{label}");
    }
}

#[test]
fn only_a_branch_under_rekall_with_a_safe_name_is_one_of_ours() {
    assert!(layout().require_branch("rekall/report").is_ok());
    for branch in ["main", "rekall/", "rekall/../main", "feature/report"] {
        assert!(layout().require_branch(branch).is_err(), "{branch}");
    }
}

#[test]
fn a_folder_has_to_be_inside_the_root_and_not_the_root_itself() {
    assert!(layout().require_folder(Path::new("/data/worktrees/vega/report")).is_ok());
    for folder in ["/data/worktrees", "/data/other/report", "/data/worktrees/../report", "/data/worktrees/vega/../../etc"] {
        assert!(layout().require_folder(Path::new(folder)).is_err(), "{folder}");
    }
}
