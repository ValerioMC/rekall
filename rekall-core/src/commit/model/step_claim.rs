pub struct StepClaim<'a> {
    pub project_label: Option<&'a str>,
    pub task_code: &'a str,
    pub step: &'a str,
    pub outcome: Option<&'a str>,
    pub report: Option<&'a str>,
    pub message: Option<&'a str>,
}
