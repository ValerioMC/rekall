# Write safety and connections

`rekall-mcp` has no controller, no `CompanyService`, `ProjectService` or `TaskService` and no
`DocumentService` on its classpath: they live in `rekall-console` for exactly that reason, and
anything that writes the catalog belongs there. The write services it can reach are narrow by
construction. `WrapupService` takes a task and a body and can do nothing else: no record created,
renamed or deleted, no note touched. `TaskStepService.transition` takes a task, a step and a
target state, and refuses `DONE` and any step a person has already accepted; it cannot add,
reorder or remove a step, and the console's `edit` is still the only path to `DONE`.
`TaskStepService.propose` can only append one step, open or as a draft. `FlowProposalService.propose` can
only lay a flow onto a task with no steps, and the routes it asks `StepLinkService.propose_in` for are
refused unless both ends are drafts, so no released step's routes change: a ready flow is laid as
drafts and released inside the same transaction. `TaskAuthoringService.create` does the same for the
task it has just created. None of them promotes a draft that was already on a task. The only deletes a session can make are
`FlowProposalService.replan` and `TaskStepService.remove`, and both stop at `TaskStepState::unstarted`:
a draft or an open step. A running, claimed or done step is refused by name, so a session never discards
a run, work waiting for review or work a person accepted; a replan checks its plan whole before it deletes
anything and runs in one transaction. `NoteService` (rekall-core) can only create a note and attach it to the one task it was
written for: it never loads a note by id, so it cannot edit, detach or delete one, and
`DocumentService`, which can, stays in `rekall-console`. `FeedbackService` (rekall-core) can only move a `PENDING` or `RUNNING` entry of one task's thread to `APPLIED` with a short reply, naming the `seq`, and give unanswered `RUNNING` entries back to `PENDING` when a session ends: it has no way to create, edit or delete a revision, and a call that names an unknown or already applied entry writes nothing. `CommitReferenceService` and `AutoCommitService` log and make commits in the project's own
folder. `InventoryRecordingService` replaces one project's dependency rows and drops the exposures they
no longer declare; `VulnerabilityExposureService.assess` writes a verdict on an exposure a scan already
found and refuses an advisory that does not reach the project. Neither reaches the advisory database:
the scan, which does, is not part of `Services` and is built in `rekall-app` for the console and the
schedule, and the fix task is created by `VulnerabilityFixService` in `rekall-console`. Every read runs in a
read-only transaction. `McpTool.writes()` is declared
rather than inferred, and the startup log names the write surface out loud.

The commands that gate a claim (`done_check`) are written by `CheckService` in `rekall-console` and by nothing else. `rekall-mcp` lists `rekall-core` as its only Rekall dependency, so it cannot name that service, and no tool creates or edits a check: a session able to write the command that verifies its own work could write one that always passes.

Running those commands is a port. `CheckExecutionService` (`rekall-core/src/check/`, in `Services::check_execution`) reads a step's checks (the project's first, then the step's), runs them one at a time through `CheckRunner` and stops at the first that fails; each run is saved to `done_check_result`, the failing one included, and the answer is a `CheckReport` (`NoChecks`, `Passed`, `Failed`). A check's folder is the project's folder, or a folder inside it: an absolute path or `..` is refused with a 400. `ShellCheckRunner` (`rekall-infra/src/check/`) runs `/bin/sh -c` in the environment of the user's login shell, in its own process group so a timeout ends the command and its children. It keeps the last 60 lines of the combined output (at most 4000 characters) and never logs them, since they may hold secrets; the log carries the label, the exit code and the duration.

Connection credentials never reach `rekall-mcp`. A GitLab or GitHub token, a Kubernetes kubeconfig or a Jira
credential (`JiraCredential`: a Jira Cloud email and API token, kept as the `email:token` pair Basic
authentication sends, or a Data Center personal access token sent as Bearer, kept as `pat:token`; which
one it is decides the site's `JiraDeployment`) lives in
Rekall's own credential file (`FileSecretStore` in `rekall-infra`: `secrets.json` under `rekall.home`, one
account per connection's `secret_ref`, mode 0600, not encrypted, outside the database and its backups),
behind the `SecretStore` trait in `rekall-core/src/connection/`. The file never prompts, so a terminal's
helpers read it on every `kubectl`, `git` or API call at no cost. Credentials an earlier version kept in the OS
credential store (`KeychainSecretStore`, over `keyring`, service `Rekall-connections`) are moved by a
`MigratingSecretStore`: an account the file does not hold is read once from the keychain, through a
`RememberingSecretStore`, and copied into the file; a put goes to the file only; a removal reaches both, so the
old copy cannot come back. `rekall-app` builds the store and hands it to `ApiState` (through `ApiPorts`) and the
`CredentialBroker` only: it is not part of `Services`, which is what the MCP tools are built from, and
`rekall-mcp/tests/integration/credential_isolation.rs` fails if `rekall-mcp` or `Services` ever names
it or a type that carries a credential. A credential in memory is a `Secret`: it prints as `***`, has
no `Serialize` and is wiped on drop. A kubeconfig is parsed by `Kubeconfig`, which refuses a user
that runs a command (`exec`, `auth-provider`), a username and password, a server that is not
`https://` or skips TLS verification, a `proxy-url` that is not `http://` or `https://` or carries a
username or password, and a file path that is relative or unreadable, and whose parse errors name a
line, never the text. The chosen context is kept as `KubeAccess::to_kubeconfig`: that context alone,
its `proxy-url` kept, with every referenced file inlined. On Linux the store is the kernel keyring,
which forgets its entries when the session ends.

The console's write path for connections is `rekall-console/src/connection/`, one service per entity,
each taking a command and answering a view that the controllers in `rekall-http/src/connection/` map
from the request and to the response:
`ConnectionService` (the workspace's connections and the companies each is shared with; `set_companies`
refuses with a 409 naming the count while a project of a company being removed still binds the
connection, so nothing is unbound silently; deleting one removes its credential from the store
after the row is gone, logging a failure), `ConnectionCredentialService` (the only code that stores,
replaces or clears a credential), `ProjectConnectionService` (bindings; a project binds only a
connection shared with its own company (400 otherwise, saying to share it first), the same connection and target twice is a 409, a typed label already
used in the project is a 409, and without one the connection's label takes the first free `-2`, `-3`),
`RepositoryDiscoveryService` (further down), `ConnectionAccessQueryService` (the access log, newest first) and
`KubeconfigPreviewService` (a kubeconfig's contexts with why each can or cannot be imported, storing
nothing). Credentials go in through `PUT /api/connections/{id}/credential`, which takes `token` (GitLab, GitHub, a Grafana service account),
`kubeconfig` and `context` (Kubernetes) or `token` with an `email` on Jira Cloud and without one on Jira
Data Center, and answers the connection with `credentialSetAt`. The only route that returns one is
`GET /api/connections/{id}/credential`, served by `ConnectionCredentialRevealService` (the write service
stays write-only): it answers `token`, `kubeconfig` and, on Jira Cloud, `email`, or a 404 when none is stored, and
the console calls it only when the person presses the eye or the copy button on a connection's credential,
keeping the answer in the component until it is hidden, replaced or the view closes. Once shown, the value
can be edited in place and stored through the same `PUT`; a Kubernetes edit keeps the stored context. Like the rest of the
console API it answers any local caller, so a session that curls it by hand reads the credential; the deny
rules below do not cover it. The store is written before the row, so a failed write leaves the row
saying no credential is set; clearing empties the store before the row, so the row never claims less
than the store holds. A Kubernetes connection takes its server and context from the imported
kubeconfig and refuses a typed URL; GitLab, GitHub (`https://github.com` or an Enterprise Server), Jira and Grafana need theirs; a connection's kind is fixed once created.
Only a Jira or Grafana connection takes a `proxyUrl`: an `http://` or `https://` address with no user or password in
it (it is stored in the row and shown, not kept in the keychain), refused without being echoed; a blank
one clears it. Only a Grafana connection takes `acceptSelfSigned`, a flag; the other kinds refuse it when it is true. A GitLab or GitHub connection may name a `rootPath`, the group, namespace or owner its token covers, stored
without a slash at either end; the other kinds refuse it. A label is unique across the workspace. The routes are
`GET`/`POST /api/connections` (`POST` takes `companyIds`, the companies it is shared with, and no company
is needed to create one), `GET /api/companies/{id}/connections` (the connections shared with a company: the
ones its projects can bind), `PUT /api/connections/{id}/companies` (replaces `companyIds`),
`GET`/`PUT`/`DELETE /api/connections/{id}` (`GET` lists the companies it is shared with and the projects
that bind it, each with its company; `PUT` never changes the companies; `DELETE` answers `unboundProjects`), `PUT`/`DELETE
/api/connections/{id}/credential`, `POST /api/connections/kubeconfig/contexts`, `GET
/api/connections/{id}/access?offset=&limit=`, `GET`/`POST /api/projects/{id}/connections` (`POST` binds
`connectionId` with an optional `label` and `target`), `PUT`/`DELETE
/api/projects/{id}/connections/{bindingId}` (`PUT` sets the binding's `target`, and its `label` when one
is sent) and `POST /api/projects/{id}/connections/discover`. A connection's `projectCount` counts each
project once; its detail lists a row per binding. Every change publishes a `connection` frame naming
the companies concerned (`companyIds`).

`RepositoryDiscoveryService` (`rekall-console/src/connection/`) binds a project's repositories in one go.
It walks the project's folder (`repository_folders.rs`: a folder holding `.git` is a repository and is
not searched further, hidden folders and `node_modules` are passed over, four levels deep), reads each
one's `origin` through `GitRepositoryInspector::origin_url`, all before its transaction opens, then
settles each repository in path order:

- `RepositoryOrigin` reads `https://`, `ssh://` and `git@host:path` origins as a host and a project path,
  dropping any user or token, and matches a GitLab or GitHub connection shared with the company on the host (port ignored;
  an instance under a sub-path drops it from an HTTPS origin). A connection with a `root_path` covers only
  the repositories under it, whole segments, ignoring case; the longest root path wins, then one with a
  credential (`rule/repository_match_rules.rs`).
- No origin is `NO_ORIGIN`, no GitLab or GitHub connection at the host `NO_CONNECTION`, one with no credential
  `NO_CREDENTIAL`: nothing is bound for those, since the token is what the binding reaches.
- A project path the project already binds on that connection is `ALREADY_BOUND`, whether by hand, by an
  earlier run or by a second clone found earlier in the same run. Otherwise it is `BOUND` under the
  folder's name as a label (the first free `-2`, `-3`; the connection's label when the name leaves no slug).
- The first folder of each project path in a run gets a component through
  `ProjectComponentCatalogService::add_for_folder_in`, named after the folder (the first free `-2`
  ignoring case), unless a component already has that folder. A second clone gets none.

The answer lists every repository with its outcome, `host/path` (never the remote URL), the binding and
the component made, and a project with no folder, or one that is not on this machine, is a 400.

`JiraIssueService` (`rekall-console/src/connection/`) is how a project turns Jira issues into tasks, through
its binding of a Jira connection whose target is the Jira project key. `GET
/api/projects/{id}/connections/{connectionId}/jira-issues` lists the project's 50 most recently updated
issues not yet done, each with its link and the task it was imported as; `POST .../jira-issues/import`
takes `keys` and creates one task per issue in one transaction (`TaskService::create_in`). The rules are
in `jira_rules.rs`:

- An issue's task is labelled with its key lower-cased (`eng-12`), which is also how an imported issue is
  recognised: importing it again adds nothing, and a hand-made task already holding that label counts as
  its import.
- The title is the summary (cut to 200 characters, the key when blank); the description is a link back to
  the issue followed by its description, read from Atlassian's document format as markdown
  (`rekall-infra/src/jira/atlassian_document.rs`) on Jira Cloud and kept as Jira wrote it on
  Data Center, whose v2 API answers it as text.
- A key that is not `<PROJECT>-<number>` of the bound project, more than 50 keys, a binding with no project
  key, a connection of another kind or one with no credential is refused (400) before Jira is asked.
- Jira rejecting the credential (401/403) or not answering is a 409 naming why, with Jira's own message
  when it gave one.

Jira is reached through the `JiraIssues` trait in `rekall-core/src/connection/`, implemented by
`HttpJiraIssues` in `rekall-infra` (`POST /rest/api/3/search/jql` on Jira Cloud, `POST
/rest/api/2/search` on Data Center, the first page only), or a fake
through `StartOptions::jira_issues`. It is asked before any transaction opens. Both it and the check
take a `JiraSite`, the site URL and the connection's proxy, and go through the proxy when there is one. The Grafana check takes a `GrafanaInstance`: the URL, the proxy and whether the instance's self-signed certificate is accepted (certificate verification is then off, which also lets an expired one or one for another host through).

`POST /api/connections/{id}/check` asks the connection's server whether its stored credential still
works, before a session finds out mid-deploy. `ConnectionCheckService` (`rekall-core/src/connection/`)
reads the credential through the store and hands it to a `ConnectionProbe`: `HttpConnectionProbe` in
`rekall-infra/src/probes/`, because asking is HTTP to the outside, or a fake through
`StartOptions::connection_probe`.
GitLab is asked `GET /api/v4/user` with the token, GitHub `GET /user` on its API root (`github_api_root`:
`api.github.com`, or `/api/v3` on an Enterprise Server) with a Bearer token and a `User-Agent`, Jira `GET /rest/api/3/myself` (Cloud) or `GET /rest/api/2/myself` (Data Center) with the credential's header, Grafana `GET /api/user` with a Bearer token; a cluster is asked `/version` and a
`SelfSubjectReview`, through the kubeconfig's `proxy-url` when it has one, trusted through its CA
and with a client certificate as the TLS identity, falling back to an authenticated `GET /api` on a cluster too old for the review. The
answer (`ok`, a sentence naming the identity, the rejection status or why the server could not be
asked, with the underlying cause such as a timeout) is shown and never stored, and a connection with no credential is answered without asking
anyone.

A terminal with a credential session also starts `claude` with `--settings` pointing at the
session's `claude-settings.json`, whose `permissions.deny` (`rekall-infra/src/connection/guard_rails.rs`)
closes the session folder to the file tools (`Read`/`Edit` with an absolute `//` rule on the folder's
real path) and denies dumping the environment (`env`, `printenv`, `export -p`, `set`), `git
credential`, `kubectl config view --raw`, running the plugin or helper by hand, `cat`/`head`/`tail`/
`less`/`more` on the folder, `curl`/`wget` to the credentials endpoint, and the `security` keychain
commands. They add to the person's own settings rather than replacing them, and they hold under
`--dangerously-skip-permissions`: checked against Claude Code 2.1.306, where a `--settings` deny on
`printenv` was refused in bypass mode while the same call ran without the rule, and an absolute
`Read(//<folder>/**)` rule was refused too. They are prefix matches against an accidental read, not
a sandbox: a command shaped differently (`kubectl config view --minify --raw`, a script that calls
the endpoint) is not caught, and a session set on extracting a credential can still read
`REKALL_CREDENTIAL_TOKEN` through a command the rules do not name.

### The guard command

Two rules that a session is only asked to follow have a program behind them: on an `auto-commit`
project nothing runs `git commit`, `git push` or `git add` by hand, and nothing reads, copies or
searches where credentials are kept. `rekall-server guard before-tool` and `guard before-stop`
(`rekall-app/src/guard_command/`) are what Claude Code runs as a `PreToolUse` and a `Stop` hook:

1. Claude Code writes the event to standard input (`cwd`, `tool_name`, `tool_input.command`, `file_path`
   or `path`, and `stop_hook_active` on a `Stop`). The parser is `HookInput`.
2. The command asks the running Rekall for `GET /api/guard/context?folder=<cwd>` (one request,
   one-second timeout, the port from `AppConfig`). `GuardContextService` (`rekall-console`) answers
   with the project that holds the folder (the innermost wins), its `auto_commit`, the first running
   step as a number and a task anchor, and the credential locations to protect.
3. `GuardService::decide` (`rekall-core/src/guard/`) returns `Allow` or `Block { reason }`. Exit code 0
   lets the action go ahead; exit code 2 blocks it and the reason, on standard error, goes to the model.

The credential locations are `<rekall.home>/secrets.json`, `~/.claude/.credentials.json` and the
`/api/terminal/` prefix of the helper endpoint, built by `rekall-infra::secret::protected_references`
from the same constants the stores and the broker use. A tool call that names one is blocked on every project.

The command lets the action through when it cannot decide: an input it cannot parse, a command line with an
unclosed quote, a Rekall that does not answer. It writes the reason to `<rekall.home>/guard.log`. A guard that failed
closed would stop every session while the app is shut. A `Stop` that follows a block (`stop_hook_active`) is
let through, so the session can explain why a step cannot be claimed instead of looping. The hooks reach
only the first running step of a project, so a second session in the same project is blocked from
stopping while another session's step is running. They are not a sandbox: a command built from a variable
or a script that runs git is not recognised.

Settings has a **Protections** section that writes the hooks into `~/.claude/settings.json`
(`GuardInstaller`, a port of `rekall-core`; `GuardHookInstaller` in `rekall-infra/src/installer/`;
`GET /api/guard/status`, `POST /api/guard/install`, `POST /api/guard/uninstall`). It adds one
`PreToolUse` group (matcher `Bash|Read|Edit|Write`) running `<this executable> guard before-tool` and one
`Stop` group running `guard before-stop`. An entry is Rekall's when its command ends in one of those two
phrases, so installing twice leaves one of each (and replaces the path after the app moved) and removing
takes out only those. Every other key and hook is kept, in its order. A file that is not JSON, or whose
`hooks` have another shape, is refused with a 400 and not touched; the first write keeps
`settings.json.rekall-backup`; writes go through a temporary file and a rename. The hook runs the same
executable as the app, so `rekall-server` and the desktop app both answer `guard`.

The residual risk is real and small: Claude can overwrite one task's wrapup with something
wrong, or with something that replaces a correction made by hand, and it can mark a step running
or claimed when it is not. Nothing keeps the text a wrapup replaces, so the tool's answer announces
a replaced hand edit and the session tells the person, who writes the correction again. The third a person corrects with one tick in the console. It can also add a note
nobody wanted, which a person deletes from the note's pane. What it cannot do is tick a step
done, change or lose a note, move a task or delete anything.

