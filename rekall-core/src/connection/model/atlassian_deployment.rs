/// Which edition of Jira or Confluence a site runs, as its credential tells: they sign in differently and serve different
/// REST API versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtlassianDeployment {
    Cloud,
    DataCenter,
}
