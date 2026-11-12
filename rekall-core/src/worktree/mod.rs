//! A second working folder of a project's repository, on a branch of its own, so two sessions can
//! work two tasks at once without overwriting each other's files. The names and the safety rules
//! are here; git is reached through `WorktreeHost`.

mod model;
mod port;
mod rule;
mod service;

pub use model::branch_review::BranchReview;
pub use model::merge_outcome::MergeOutcome;
pub use model::orphan_report::OrphanReport;
pub use model::pull_request_flow_site::PullRequestFlowSite;
pub use model::worktree::Worktree;
pub use model::worktree_status::WorktreeStatus;
pub use port::worktree_host::WorktreeHost;
pub use rule::pull_request_flow_rules::{require_ground, require_host_kept};
pub use rule::worktree_layout::{WorktreeLayout, BRANCH_PREFIX};
pub use rule::worktree_listing::parse_listing;
#[cfg(any(test, feature = "testing"))]
pub use port::scripted_worktree_host::ScriptedWorktreeHost;
pub use service::orphan_reclaim_service::OrphanReclaimService;
pub use service::pull_request_flow_service::PullRequestFlowService;
pub use service::task_worktree_service::{Merge, Release, TaskWorktreeService};
