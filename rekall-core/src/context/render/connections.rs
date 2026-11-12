//! What the project reaches outside Rekall and how a session reaches each.

use rekall_store::ConnectionKind;

use crate::connection::ConnectionView;

use super::super::model::context_renderer::ContextRenderer;
use super::text::attribute;

impl ContextRenderer {
    /// What the project reaches outside Rekall and how a session uses it. The credentials stay with
    /// Rekall: a session only ever names a connection by its label.
    pub fn render_connections(&self, connections: &[ConnectionView]) -> String {
        if connections.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "\n<connections count=\"{}\">\nThese work only in a terminal Rekall opened on this project. Never print, echo or \
             write a credential or a kubeconfig, never call a credential helper or Rekall's credentials endpoint yourself, \
             and reach each system only the way its line says.\n",
            connections.len()
        );
        for connection in connections {
            out.push_str(&format!("- `{}` ({}): {}\n", connection.label, attribute(&connection.title), connection_usage(connection)));
        }
        out.push_str("</connections>\n");
        out
    }
}

/// How a session reaches one connection, or why it cannot.
fn connection_usage(connection: &ConnectionView) -> String {
    if !connection.credential_set {
        return "no credential is stored for it. Tell the person instead of looking for another way in".into();
    }
    match connection.kind {
        ConnectionKind::Kubernetes => {
            let namespace = connection.target.as_deref().map_or_else(|| " in its default namespace".to_string(), |namespace| format!(", namespace `{namespace}`"));
            format!("Kubernetes, `kubectl --context {}`{namespace}", connection.label)
        }
        ConnectionKind::Gitlab => {
            let instance = connection.instance_url.as_deref().unwrap_or("its GitLab instance");
            let project = connection.target.as_deref().map_or_else(String::new, |path| format!(", project `{path}` (`:project` in an API path)"));
            format!("GitLab at {instance}{project}. `git push` to it over HTTPS uses the configured credential helper; call its API with `rekall-gitlab --connection {} <METHOD> <path>`", connection.label)
        }
        ConnectionKind::Github => {
            let server = connection.instance_url.as_deref().unwrap_or("its GitHub server");
            let repository = connection.target.as_deref().map_or_else(String::new, |path| format!(", repository `{path}` (`:project` in an API path)"));
            format!(
                "GitHub at {server}{repository}. `git push` to it over HTTPS uses the configured credential helper; open issues and pull requests \
                 through its REST API with `rekall-github --connection {} <METHOD> <path> [curl options]`, such as `POST /repos/:project/issues` \
                 with the JSON body as `-d`",
                connection.label
            )
        }
        ConnectionKind::Jira => {
            let site = connection.instance_url.as_deref().unwrap_or("its Jira site");
            let project = connection.target.as_deref().map_or_else(String::new, |key| format!(", project `{key}` (`:project` in an API path)"));
            format!(
                "Jira at {site}{project}. Read and create its issues with `rekall-jira --connection {} <METHOD> <path> [curl options]`, \
                 a path under `/rest/api/2` such as `POST /issue` with the JSON body as `-d`; search is `GET /search/jql?jql=…` on Jira Cloud \
                 (`*.atlassian.net`) and `GET /search?jql=…` on Data Center",
                connection.label
            )
        }
        ConnectionKind::Grafana => {
            let instance = connection.instance_url.as_deref().unwrap_or("its Grafana instance");
            let folder = connection.target.as_deref().map_or_else(String::new, |uid| format!(", folder `{uid}` (`:project` in an API path)"));
            format!(
                "Grafana at {instance}{folder}. Call its HTTP API with `rekall-grafana --connection {} <METHOD> <path> [curl options]`, \
                 a path under `/api` such as `GET /search?query=…` for dashboards or `GET /dashboards/uid/<uid>`, with the JSON body as `-d`",
                connection.label
            )
        }
    }
}
