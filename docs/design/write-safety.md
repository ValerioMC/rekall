# Write safety and connections

`rekall-mcp` has no controller, no `CompanyService`, `ProjectService` or `TaskService` and no
`DocumentService` on its classpath: they live in `rekall-console` for exactly that reason, and
anything that writes the catalog belongs there. The write services it can reach are narrow by
construction. `WrapupService` takes a task and a body and can do nothing else: no record created,
renamed or deleted, no note touched. `TaskStepService.transition` takes a task, a step and a
target state, and refuses `DONE` and any step a person has already accepted; it cannot add,
reorder or remove a step, and the console's `edit` is still the only path to `DONE`.
`TaskStepService.propose` can only append a draft, which no session reads until a person promotes
it. `FlowProposalService.propose` can only lay drafts onto a task with no steps, and the routes it
asks `StepLinkService.propose_in` for are refused unless both ends are drafts, so no released
step's routes change. `NoteService` (rekall-core) can only create a note and attach it to the one task it was
written for: it never loads a note by id, so it cannot edit, detach or delete one, and
`DocumentService`, which can, stays in `rekall-console`. `CommitReferenceService` and `AutoCommitService` log and make commits in the project's own
folder. `InventoryRecordingService` replaces one project's dependency rows and drops the exposures they
no longer declare; `VulnerabilityExposureService.assess` writes a verdict on an exposure a scan already
found and refuses an advisory that does not reach the project. Neither reaches the advisory database:
the scan, which does, is not part of `Services` and is built in `rekall-app` for the console and the
schedule, and the fix task is created by `VulnerabilityFixService` in `rekall-console`. Every read runs in a
read-only transaction. `McpTool.writes()` is declared
rather than inferred, and the startup log names the write surface out loud.

Connection credentials never reach `rekall-mcp`. A GitLab token, a Kubernetes kubeconfig or a Jira
credential (`JiraCredential`: a Jira Cloud email and API token, kept as the `email:token` pair Basic
authentication sends, or a Data Center personal access token sent as Bearer, kept as `pat:token`; which
one it is decides the site's `JiraDeployment`) lives in
the OS credential store (`KeychainSecretStore` in `rekall-infra`, over `keyring`, service
`Rekall-connections`, one account per connection's `secret_ref`), behind the `SecretStore` trait in
`rekall-core/src/connection/`. `rekall-app` wraps it in a `RememberingSecretStore`, which reads each account from the keychain
once and answers later reads from memory until the account is written or removed: the Keychain may
prompt on every read, and a terminal's helpers read on every `kubectl`, `git` or API call. An edit made
in Keychain Access is seen after a restart. `rekall-app` builds the store and hands it to `ApiState` (through `ApiPorts`) and the
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
`ConnectionService` (a company's connections; deleting one removes its credential from the store
after the row is gone, logging a failure), `ConnectionCredentialService` (the only code that stores,
replaces or clears a credential), `ProjectConnectionService` (bindings; a project binds only a
connection of its own company, the same connection and target twice is a 409, a typed label already
used in the project is a 409, and without one the connection's label takes the first free `-2`, `-3`),
`RepositoryDiscoveryService` (further down), `ConnectionAccessQueryService` (the access log, newest first) and
`KubeconfigPreviewService` (a kubeconfig's contexts with why each can or cannot be imported, storing
nothing). Credentials are write-only: `PUT /api/connections/{id}/credential` takes `token` (GitLab),
`kubeconfig` and `context` (Kubernetes) or `token` with an `email` on Jira Cloud and without one on Jira
Data Center, and answers the connection with `credentialSetAt`, and no
endpoint returns a credential. The store is written before the row, so a failed write leaves the row
saying no credential is set; clearing empties the store before the row, so the row never claims less
than the store holds. A Kubernetes connection takes its server and context from the imported
kubeconfig and refuses a typed URL; GitLab and Jira need theirs; a connection's kind is fixed once created.
Only a Jira connection takes a `proxyUrl`: an `http://` or `https://` address with no user or password in
it (it is stored in the row and shown, not kept in the keychain), refused without being echoed; a blank
one clears it. The routes are
`GET`/`POST /api/companies/{id}/connections`, `GET`/`PUT`/`DELETE /api/connections/{id}` (`GET` lists
the projects that bind it, `DELETE` answers `unboundProjects`), `PUT`/`DELETE
/api/connections/{id}/credential`, `POST /api/connections/kubeconfig/contexts`, `GET
/api/connections/{id}/access?offset=&limit=`, `GET`/`POST /api/projects/{id}/connections` (`POST` binds
`connectionId` with an optional `label` and `target`), `PUT`/`DELETE
/api/projects/{id}/connections/{bindingId}` (`PUT` sets the binding's `target`, and its `label` when one
is sent) and `POST /api/projects/{id}/connections/discover`. A connection's `projectCount` counts each
project once; its detail lists a row per binding. Every change publishes a `connection` frame naming
the company.

`RepositoryDiscoveryService` (`rekall-console/src/connection/`) binds a project's repositories in one go.
It walks the project's folder (`repository_folders.rs`: a folder holding `.git` is a repository and is
not searched further, hidden folders and `node_modules` are passed over, four levels deep), reads each
one's `origin` through `GitRepositoryInspector::origin_url`, all before its transaction opens, then
settles each repository in path order:

- `RepositoryOrigin` reads `https://`, `ssh://` and `git@host:path` origins as a host and a project path,
  dropping any user or token, and matches a GitLab connection of the company on the host (port ignored;
  an instance under a sub-path drops it from an HTTPS origin). One with a credential is preferred.
- No origin is `NO_ORIGIN`, no GitLab connection at the host `NO_CONNECTION`, one with no credential
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
take a `JiraSite`, the site URL and the connection's proxy, and go through the proxy when there is one.

`POST /api/connections/{id}/check` asks the connection's server whether its stored credential still
works, before a session finds out mid-deploy. `ConnectionCheckService` (`rekall-core/src/connection/`)
reads the credential through the store and hands it to a `ConnectionProbe`: `HttpConnectionProbe` in
`rekall-infra/src/probes/`, because asking is HTTP to the outside, or a fake through
`StartOptions::connection_probe`.
GitLab is asked `GET /api/v4/user` with the token, Jira `GET /rest/api/3/myself` (Cloud) or `GET /rest/api/2/myself` (Data Center) with the credential's header; a cluster is asked `/version` and a
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

The residual risk is real and small: Claude can overwrite one task's wrapup with something
wrong, or with something that replaces a correction made by hand, and it can mark a step running
or claimed when it is not. Nothing keeps the text a wrapup replaces, so the tool's answer announces
a replaced hand edit and the session tells the person, who writes the correction again. The third a person corrects with one tick in the console. It can also add a note
nobody wanted, which a person deletes from the note's pane. What it cannot do is tick a step
done, change or lose a note, move a task or delete anything.

