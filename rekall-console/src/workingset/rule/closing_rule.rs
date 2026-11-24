use rekall_kernel::Id;

/// Where a project lands when `closed` is closed: the task opened after it, else the one opened
/// before it, else the project's task list (`None`). `open_in_project` is in the order opened.
pub(in crate::workingset) fn landing_after_closing(open_in_project: &[Id], closed: Id) -> Option<Id> {
    let index = open_in_project.iter().position(|task| *task == closed)?;
    open_in_project.get(index + 1).or_else(|| index.checked_sub(1).and_then(|before| open_in_project.get(before))).copied()
}

#[cfg(test)]
#[path = "../../../tests/unit/workingset/rule/closing_rule_tests.rs"]
mod tests;
