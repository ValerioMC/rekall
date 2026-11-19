use super::*;

fn ground(has_folder: bool, has_branch: bool, task_manager: Option<ConnectionKind>) -> Result<()> {
    require_ground(PullRequestFlowSite { has_folder, has_branch }, task_manager)
}

#[test]
fn a_folder_a_branch_and_a_github_or_gitlab_task_manager_allow_the_flow() {
    assert!(ground(true, true, Some(ConnectionKind::Github)).is_ok());
    assert!(ground(true, true, Some(ConnectionKind::Gitlab)).is_ok());
}

#[test]
fn a_missing_folder_branch_or_task_manager_refuses_the_flow_naming_it() {
    let missing_folder = ground(false, true, Some(ConnectionKind::Github)).unwrap_err().to_string();
    let missing_branch = ground(true, false, Some(ConnectionKind::Github)).unwrap_err().to_string();
    let missing_manager = ground(true, true, None).unwrap_err().to_string();
    assert!(missing_folder.contains("folder"), "{missing_folder}");
    assert!(missing_branch.contains("branch"), "{missing_branch}");
    assert!(missing_manager.contains("task manager"), "{missing_manager}");
}

#[test]
fn a_jira_task_manager_refuses_the_flow() {
    let refusal = ground(true, true, Some(ConnectionKind::Jira)).unwrap_err().to_string();
    assert!(refusal.contains("GitHub or GitLab"), "{refusal}");
}

#[test]
fn a_flow_that_is_on_keeps_a_github_or_gitlab_task_manager_and_refuses_jira_or_none() {
    assert!(require_host_kept(Some(ConnectionKind::Github)).is_ok());
    assert!(require_host_kept(Some(ConnectionKind::Gitlab)).is_ok());
    assert!(require_host_kept(Some(ConnectionKind::Jira)).is_err());
    assert!(require_host_kept(None).is_err());
}
