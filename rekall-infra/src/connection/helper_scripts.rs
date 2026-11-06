//! The shell helpers a credential session writes. Each asks Rekall's endpoint for one credential
//! with the terminal's token, sent on stdin so it never shows in a process listing, and hands the
//! answer straight to the tool that asked. None holds a credential.

use rekall_core::claude::TerminalConnection;
use rekall_core::connection::github_api_root;

pub(super) const KUBE_PLUGIN: &str = "rekall-kube-credential";
pub(super) const GIT_HELPER: &str = "rekall-git-credential";

/// Fetches `$1` with the token as a header read from stdin; prints the body, or exits non-zero.
const FETCH: &str = r#"rekall_fetch() {
  printf 'X-Rekall-Credential-Token: %s\n' "$REKALL_CREDENTIAL_TOKEN" |
    curl --silent --show-error --fail-with-body --noproxy '*' -H @- "$1"
}
"#;

const HEADER: &str = "#!/bin/sh\n# Written by Rekall for one terminal and removed when it ends. It holds no credential.\n";

/// kubectl's exec plugin: `rekall-kube-credential <label>` prints an `ExecCredential`.
pub(super) fn kube_plugin(endpoint: &str) -> String {
    format!(
        "{HEADER}{FETCH}answer=$(rekall_fetch {}\"$1?for=kubectl\") || {{ printf 'rekall: %s\\n' \"$answer\" >&2; exit 1; }}\nprintf '%s\\n' \"$answer\"\n",
        quoted(&format!("{endpoint}/"))
    )
}

/// git's credential helper: answers `get` with the token; `store` and `erase` have nothing to do.
pub(super) fn git_helper(endpoint: &str) -> String {
    format!(
        "{HEADER}{FETCH}[ \"$2\" = get ] || exit 0\ncat > /dev/null\nanswer=$(rekall_fetch {}\"$1?for=git\") || {{ printf 'rekall: %s\\n' \"$answer\" >&2; exit 1; }}\nprintf '%s\\n' \"$answer\"\n",
        quoted(&format!("{endpoint}/"))
    )
}

/// What tells one API wrapper from another. Each is `<name> --connection <label> <METHOD> <path>
/// [curl options]`: a call under `api_base` with the credential's header, `:project` in the path
/// standing for the binding's target, through the connection's proxy when it names one.
pub(super) struct ApiWrapper {
    pub(super) name: &'static str,
    system: &'static str,
    /// The API's base URL for the server URL a connection names.
    api_base: fn(&str) -> String,
    /// GitLab takes a project path as one escaped segment; GitHub's `owner/repo` stays two.
    escapes_target: bool,
    purpose: &'static str,
    /// Further curl options every call carries, each followed by a space.
    headers: &'static str,
}

/// GitLab's API with the `PRIVATE-TOKEN` header, `:project` the project path.
pub(super) const GITLAB_API: ApiWrapper =
    ApiWrapper { name: "rekall-gitlab", system: "GitLab", api_base: gitlab_api_base, escapes_target: true, purpose: "gitlab", headers: "" };

/// GitHub's REST API with a bearer token and the API version it is written against, `:project` the `owner/repo`.
pub(super) const GITHUB_API: ApiWrapper = ApiWrapper {
    name: "rekall-github",
    system: "GitHub",
    api_base: github_api_root,
    escapes_target: false,
    purpose: "github",
    headers: "-H 'Accept: application/vnd.github+json' -H 'X-GitHub-Api-Version: 2022-11-28' ",
};

/// Jira's REST API v2, which Cloud and Data Center both serve, with the credential's header and JSON
/// both ways, `:project` the project key.
pub(super) const JIRA_API: ApiWrapper = ApiWrapper {
    name: "rekall-jira",
    system: "Jira",
    api_base: jira_api_base,
    escapes_target: true,
    purpose: "jira",
    headers: "-H 'Accept: application/json' -H 'Content-Type: application/json' ",
};

/// Grafana's HTTP API under `/api` with the service account token as a bearer and JSON both ways,
/// `:project` the folder UID.
pub(super) const GRAFANA_API: ApiWrapper = ApiWrapper {
    name: "rekall-grafana",
    system: "Grafana",
    api_base: grafana_api_base,
    escapes_target: true,
    purpose: "grafana",
    headers: "-H 'Accept: application/json' -H 'Content-Type: application/json' ",
};

pub(super) fn api_wrapper(wrapper: &ApiWrapper, endpoint: &str, connections: &[&TerminalConnection]) -> String {
    let ApiWrapper { name, system, api_base, escapes_target, purpose, headers } = wrapper;
    let mut cases = String::new();
    for connection in connections {
        let base = api_base(connection.instance_url.as_deref().unwrap_or_default());
        let target = connection.target.as_deref().unwrap_or_default();
        let project = if *escapes_target { percent_encoded(target) } else { target.to_string() };
        let proxy = connection.proxy_url.as_deref().unwrap_or_default();
        cases.push_str(&format!("  {}) base={}; project={}; proxy={} ;;\n", quoted(&connection.label), quoted(&base), quoted(&project), quoted(proxy)));
    }
    format!(
        "{HEADER}{FETCH}usage() {{ echo 'usage: {name} --connection <label> <METHOD> <path> [curl options]' >&2; exit 2; }}
[ \"$1\" = --connection ] && [ $# -ge 4 ] || usage
label=$2; method=$3; path=$4; shift 4
case \"$label\" in
{cases}  *) echo \"{name}: this terminal has no {system} connection labelled $label\" >&2; exit 2 ;;
esac
case \"$path\" in /*) ;; *) path=\"/$path\" ;; esac
[ -n \"$project\" ] && path=$(printf '%s' \"$path\" | sed \"s|:project|$project|g\")
header=$(rekall_fetch {}\"$label?for={purpose}\") || {{ printf '{name}: %s\\n' \"$header\" >&2; exit 1; }}
[ -n \"$proxy\" ] && set -- --proxy \"$proxy\" \"$@\"
printf '%s\\n' \"$header\" | curl --silent --show-error -X \"$method\" -H @- {headers}\"$base$path\" \"$@\"
",
        quoted(&format!("{endpoint}/"))
    )
}

fn gitlab_api_base(instance_url: &str) -> String {
    format!("{}/api/v4", instance_url.trim_end_matches('/'))
}

fn grafana_api_base(instance_url: &str) -> String {
    format!("{}/api", instance_url.trim_end_matches('/'))
}

fn jira_api_base(site_url: &str) -> String {
    format!("{}/rest/api/2", site_url.trim_end_matches('/'))
}

/// `text` as one single-quoted shell word.
fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// A target as an API path takes it, a GitLab project path in place of an id: every byte but the unreserved ones escaped.
fn percent_encoded(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (byte as char).to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/connection/helper_scripts_tests.rs"]
mod tests;
